-- Shmup simulation: player, bugs, bullets, bombs, pickups, particles.

local G = require "src.gfx"
local Balance = require "src.balance"

local World = {}
World.__index = World

local function dist2(ax, ay, bx, by)
  local dx, dy = ax - bx, ay - by
  return dx * dx + dy * dy
end

local function easeCos(t)
  t = math.max(0, math.min(1, t))
  return 0.5 - 0.5 * math.cos(t * math.pi)
end

local DEFS = {
  beetle   = { hp = 1, r = 8,  score = 100,  speed = 44, fire = 0,   w = 26, h = 26, img = "beetle" },
  moth     = { hp = 2, r = 10, score = 300,  speed = 38, fire = 2.1, w = 32, h = 28, img = "moth" },
  spider   = { hp = 5, r = 11, score = 500,  speed = 24, fire = 1.55, w = 30, h = 30, img = "spider" },
  worm     = { hp = 4, r = 10, score = 400,  speed = 34, fire = 0,   w = 28, h = 32, img = "worm" },
  nullptr  = { hp = 2, r = 9,  score = 250,  speed = 56, fire = 0,   w = 24, h = 24, img = "nullptr" },
  leak     = { hp = 6, r = 12, score = 600,  speed = 20, fire = 1.05, w = 28, h = 28, img = "leak" },
  overflow = { hp = 8, r = 13, score = 800, speed = 22, fire = 1.55, w = 30, h = 32, img = "overflow" },
  deadlock = { hp = 9, r = 13, score = 700, speed = 18, fire = 1.25, w = 32, h = 24, img = "deadlock" },
  heisen   = { hp = 3, r = 10, score = 450,  speed = 42, fire = 1.7, w = 32, h = 28, img = "moth" },
  offby1   = { hp = 3, r = 10, score = 350,  speed = 44, fire = 1.55, w = 28, h = 26, img = "offby1" },
  clippy   = { hp = 6, r = 12, score = 650,  speed = 22, fire = 1.8, w = 30, h = 30, img = "clippy" },
  lifetime = { hp = 3, r = 10, score = 500,  speed = 32, fire = 1.55, w = 28, h = 32, img = "lifetime" },
  infloop  = { hp = 8, r = 13, score = 750,  speed = 26, fire = 1.2, w = 32, h = 32, img = "infloop" },
  panic    = { hp = 1, r = 11, score = 400,  speed = 62, fire = 0,   w = 28, h = 28, img = "panic" },
  bossOverflow = { hp = 420, r = 32, score = 40000, speed = 14, fire = 0.9, w = 88, h = 88, img = "bossOverflow", boss = true, title = "STACK OVERFLOW" },
  bossDeadlock = { hp = 560, r = 34, score = 55000, speed = 12, fire = 1.0, w = 90, h = 80, img = "bossDeadlock", boss = true, title = "THREAD DEADLOCK" },
  boss     = { hp = 720, r = 36, score = 80000, speed = 14, fire = 0.85, w = 104, h = 92, img = "king", boss = true, title = "SEGFAULT" },
}

-- Claude yellow (aim), Grok red (wild), Codex green (code).
-- Tokens burn like a vibe-coding budget: when they hit 0, the agent leaves.
-- Agents are shields first: wide eat radius, slow weak shots.
local AGENT_DEFS = {
  {
    id = "claude", name = "CLAUDE", img = "claude", tag = "C",
    rate = 1.25, col = { 0.95, 0.84, 0.18 },
    tokenMax = 22, drain = 0.40, shotCost = 0.04,
    orbit = 22, spin = 2.1, eatR = 30,
  },
  {
    id = "grok", name = "GROK", img = "grok", tag = "G",
    rate = 1.05, col = { 0.95, 0.22, 0.22 },
    tokenMax = 18, drain = 0.45, shotCost = 0.03,
    orbit = 36, spin = 4.4, eatR = 38,
  },
  {
    id = "codex", name = "CODEX", img = "codex", tag = "X",
    rate = 1.15, col = { 0.22, 0.88, 0.34 },
    tokenMax = 20, drain = 0.42, shotCost = 0.03,
    orbit = 14, spin = 1.5, eatR = 28,
  },
}

local AGENT_BY_ID = {}
for _, d in ipairs(AGENT_DEFS) do
  AGENT_BY_ID[d.id] = d
end

local AGENT_PICK = {
  C = "claude",
  G = "grok",
  X = "codex",
}

local PROP_DEFS = {
  cloud  = { img = "cloud",      w = 48, h = 22, vy = 96 },
  hysan  = { img = "shopHysan",  w = 52, h = 52, vy = 40 },
  market = { img = "shopMarket", w = 48, h = 48, vy = 42 },
  ramen  = { img = "shopRamen",  w = 48, h = 48, vy = 44 },
  dimsum = { img = "shopDimsum", w = 48, h = 48, vy = 42 },
  bakery = { img = "shopBakery", w = 48, h = 48, vy = 40 },
  coffee = { img = "shopCoffee", w = 46, h = 46, vy = 44 },
  case   = { img = "shopCase",   w = 46, h = 46, vy = 46 },
}

local STREET_SHOPS = { "hysan", "market", "ramen", "dimsum", "bakery", "coffee", "case" }

local SKILL_NAME = { "PRINTLN", "TRAIT", "ASYNC", "TOKIO" }

function World.new(assets, audio, hiscore, stage, diff)
  local w = setmetatable({
    assets = assets,
    audio = audio,
    hiscore = hiscore or 50000,
    stage = stage or 1,
    difficulty = Balance.normalize(diff),
  }, World)
  w:reset(1)
  return w
end

function World:applyRank()
  local bal = Balance.get(self.difficulty)
  self.hpMul = bal.hpMul + (self.loop - 1) * 0.18 + (self.stage - 1) * 0.06
  self.spdMul = bal.spdMul + (self.loop - 1) * 0.08 + (self.stage - 1) * 0.03
  self.fireRateMul = bal.fireRateMul
  self.bulletSpdMul = bal.bulletSpdMul
  self.bossHpMul = bal.bossHpMul
  self.agentDmgMul = bal.agentDmgMul
  self.agentRateMul = bal.agentRateMul
  self.agentEatMul = bal.agentEatMul
  self.agentAutoFire = bal.agentAutoFire and true or false
  self.tokenRainEmpty = bal.tokenRainEmpty
  self.tokenRainBusy = bal.tokenRainBusy
  return bal
end

function World:reset(loop)
  self.loop = loop or 1
  self.stage = self.stage or 1
  self.difficulty = Balance.normalize(self.difficulty)
  local bal = self:applyRank()
  self.time = 0
  self.score = self.score or 0
  if loop == 1 then
    self.score = 0
    self.lives = bal.lives
    self.bombs = bal.bombs
    self.power = bal.power
    self.extendAt = 30000
    self.agents = {}
    self.stage = self.stage or 1
  end
  self.agents = self.agents or {}
  self.unwrapT = 0
  self.unsafeT = 0
  self.tokioT = 0
  self.tokioFire = 0
  self.nextId = self.nextId or 1
  self.phaseName = ""
  self.laser = nil
  self.pbeam = nil
  self.combo = 0
  self.comboT = 0
  self.shake = 0
  self.hitstop = 0
  self.bgY = 0
  self.stars = {}
  local function addStars(n, v0, v1, size, a)
    for _ = 1, n do
      local vx0, vx1 = self:viewSpan()
      self.stars[#self.stars + 1] = {
        x = love.math.random(vx0, vx1 - 1),
        y = love.math.random(0, 255),
        s = size,
        v = v0 + love.math.random() * (v1 - v0),
        a = a,
        drift = (love.math.random() - 0.5) * 6,
      }
    end
  end
  addStars(28, 6, 14, 1, 0.28)   -- far
  addStars(22, 18, 36, 1.4, 0.5) -- mid
  addStars(14, 48, 90, 2.2, 0.8) -- near
  self.pbullets = {}
  self.ebullets = {}
  self.enemies = {}
  self.particles = {}
  self.pickups = {}
  self.booms = {}
  self.floaters = {}
  self.readyT = 2.2
  self.player = {
    x = 96, y = 214, vx = 0, vy = 0,
    alive = true, inv = 3.2, flash = 0,
    fireT = 0, dying = false, dieT = 0,
  }
  self.bombing = 0
  self.bombFlash = 0
  self.over = false
  self.clear = false
  self.clearT = 0
  self.boss = nil
  self.warningT = 0
  self.kills = 0
  self.graze = 0
  self.paused = false
  self.eventI = 1
  self.script = nil -- set by stage
  self.scriptDone = false
  self.scenery = {}
  self.ghosts = {}
  self.ownRing = nil
  self.muzzle = 0
  self.propT = 0
  self.fx = {}
  self.streaks = {}
  self.flash = 0
  self.recruitFX = nil
  self.agentBoost = 0
  self.shopI = 0
  self.agentDropI = self.agentDropI or 0
  self.tokenRainT = 0
  self.tokenItems = {}
end

function World:advanceStage()
  self.stage = (self.stage or 1) + 1
  self.time = 0
  self.eventI = 1
  self.scriptDone = false
  self.clear = false
  self.clearT = 0
  self.boss = nil
  self.laser = nil
  self.pbeam = nil
  self.enemies = {}
  self.ebullets = {}
  self.pbullets = {}
  self.pickups = {}
  self.scenery = {}
  self.tokenItems = {}
  self.booms = {}
  self.warningT = 0
  self.readyT = 2.0
  self.phaseName = ""
  self.player.inv = math.max(self.player.inv, 1.8)
  self:applyRank()
  local Stage = require "src.stage"
  self.script = Stage.build(self.stage, self.loop)
end

function World:addScore(n)
  self.score = self.score + math.floor(n)
  if self.score > self.hiscore then
    self.hiscore = self.score
  end
  if self.score >= self.extendAt then
    self.lives = math.min(9, self.lives + 1)
    self.extendAt = self.extendAt + 100000
    self.audio.play("oneup")
    self:floater(self.player.x, self.player.y - 16, "1UP", G.palette.yellow)
  end
end

function World:floater(x, y, text, col)
  self.floaters[#self.floaters + 1] = { x = x, y = y, t = 0.8, text = text, col = col or G.palette.white }
end

function World:burst(x, y, n, col, spd)
  spd = spd or 80
  for i = 1, n do
    local a = love.math.random() * math.pi * 2
    local v = spd * (0.3 + love.math.random())
    self.particles[#self.particles + 1] = {
      x = x, y = y,
      vx = math.cos(a) * v, vy = math.sin(a) * v,
      life = 0.25 + love.math.random() * 0.35,
      t = 0,
      r = 1 + love.math.random() * 2,
      col = col,
    }
  end
end

function World:boom(x, y, scale)
  self.booms[#self.booms + 1] = { x = x, y = y, t = 0, scale = scale or 1 }
  self:burst(x, y, 10, G.palette.yellow, 110)
  self:burst(x, y, 8, G.palette.rust, 70)
end

function World:spawn(kind, x, y, opts)
  opts = opts or {}
  local d = DEFS[kind]
  local hp
  if d.boss then
    local loopB = 1 + ((self.loop or 1) - 1) * 0.22
    hp = math.max(1, math.floor(d.hp * (self.bossHpMul or 1) * loopB))
  else
    hp = math.max(1, math.floor(d.hp * (opts.hpMul or self.hpMul)))
  end
  local e = {
    kind = kind,
    x = x, y = y, x0 = x, y0 = y,
    hp = hp,
    r = d.r,
    score = d.score,
    t = 0,
    path = opts.path or "down",
    speed = (opts.speed or d.speed) * self.spdMul,
    vx = opts.vx or 0,
    vy = opts.vy or ((opts.speed or d.speed) * self.spdMul),
    fire = d.fire,
    fireT = opts.fireT or (0.3 + love.math.random() * 0.5),
    img = d.img,
    w = d.w, h = d.h,
    tint = opts.tint or { 1, 1, 1, 1 },
    drop = opts.drop,
    boss = d.boss or kind == "boss",
    bossTitle = d.title,
    midboss = opts.midboss,
    phase = 1,
    pt = 0,
    dead = false,
    id = self.nextId,
    visible = true,
    hpFlash = 0,
    hpShow = 0,
  }
  if opts.midboss then
    e.hp = math.floor(d.hp * (opts.hpMul or self.hpMul) * 8)
    e.r = d.r + 6
    e.score = d.score * 10
  end
  e.maxhp = e.hp
  self.nextId = self.nextId + 1
  self.enemies[#self.enemies + 1] = e
  if e.boss then
    self.boss = e
  end
  return e
end

function World:spawnV(kind, n, cx, y, gap)
  gap = gap or 16
  for i = 0, n - 1 do
    local off = (i - (n - 1) / 2) * gap
    local dy = -math.abs(off) * 0.6
    self:spawn(kind, cx + off, y + dy, { path = "down" })
  end
end

function World:spawnLine(kind, n, x0, x1, y, delayPath)
  for i = 0, n - 1 do
    local u = (n == 1) and 0.5 or (i / (n - 1))
    self:spawn(kind, x0 + (x1 - x0) * u, y - i * 12, { path = delayPath or "sine" })
  end
end

function World:spark(x, y, vx, vy, col, life, r, kind)
  if #self.particles > 360 then
    return
  end
  self.particles[#self.particles + 1] = {
    x = x, y = y, vx = vx or 0, vy = vy or 0,
    life = life or 0.22, t = 0, r = r or 1.4,
    col = col or G.palette.cyan,
    kind = kind,
  }
end

function World:popRing(x, y, col, life)
  self.fx = self.fx or {}
  self.fx[#self.fx + 1] = {
    x = x, y = y, r = 3, t = 0,
    life = life or 0.28, col = col or G.palette.cyan,
  }
end

function World:starBurst(x, y, col)
  for i = 0, 3 do
    local a = i * math.pi * 0.5 + 0.4
    self:spark(x, y, math.cos(a) * 90, math.sin(a) * 90, col, 0.2, 2)
  end
end

function World:agentChomp(ag, x, y)
  ag.chomp = 0.22
  ag.eaten = (ag.eaten or 0) + 1
  self:addScore(25)
  self:burst(x, y, 10, ag.col, 150)
  self:burst(x, y, 5, G.palette.white, 80)
  self:starBurst(x, y, ag.col)
  self:popRing(x, y, ag.col, 0.3)
  self:popRing(ag.x, ag.y, G.palette.white, 0.16)
  for _ = 1, 4 do
    self:spark(x, y, (ag.x - x) * 3, (ag.y - y) * 3, ag.col, 0.18, 2)
  end
  self.flash = math.max(self.flash, 0.07)
  self.audio.play("graze")
  if ag.eaten % 5 == 0 then
    self:floater(ag.x, ag.y - 14, "EAT", ag.col)
  end
end

function World:agentPop(x, y, col)
  col = col or G.palette.cyan
  self:burst(x, y, 18, col, 170)
  self:burst(x, y, 10, G.palette.yellow, 110)
  self:starBurst(x, y, col)
  self:popRing(x, y, col, 0.42)
  self:popRing(x, y, G.palette.white, 0.2)
  self.shake = math.max(self.shake, 3.5)
  self.hitstop = math.max(self.hitstop, 0.045)
  self.flash = math.max(self.flash, 0.1)
end

function World:tryAgentEat(bx, by)
  for _, ag in ipairs(self.agents) do
    local r = ag.eatR or 16
    if dist2(bx, by, ag.x, ag.y) <= r * r then
      self:agentChomp(ag, bx, by)
      return true
    end
  end
  return false
end

function World:pshot(x, y, vx, vy, dmg, extra)
  extra = extra or {}
  local mul = (self.unsafeT > 0) and 1.7 or 1.35
  if (self.agentBoost or 0) > 0 then
    mul = mul * 1.28
  end
  self.pbullets[#self.pbullets + 1] = {
    x = x, y = y, vx = vx, vy = vy,
    dmg = (dmg or 1) * mul, r = extra.r or 4, life = extra.life or 1.2,
    pierce = extra.pierce or (self.unwrapT > 0),
    hits = {},
    col = extra.col,
    agent = extra.agent,
    trail = extra.trail ~= false,
    homing = extra.homing,
    missile = extra.missile,
  }
end

function World:spawnProp(kind, x, y, opts)
  opts = opts or {}
  local d = PROP_DEFS[kind] or PROP_DEFS.cloud
  self.scenery[#self.scenery + 1] = {
    kind = kind or "cloud",
    img = d.img,
    x = x, y = y,
    w = d.w, h = d.h,
    vy = opts.vy or d.vy,
    rot = 0,
    vr = 0,
    sc = opts.sc or 0.5,
    a = opts.a or 0.22,
  }
end

function World:spawnCloud(x, y, opts)
  self:spawnProp("cloud", x, y, opts)
end

function World:eshot(x, y, vx, vy, r, col)
  if #self.ebullets > 120 then return end
  self.ebullets[#self.ebullets + 1] = {
    x = x, y = y, vx = vx, vy = vy, r = r or 3, life = 4, grazed = false,
    col = col,
  }
end

function World:aimed(x, y, n, spread, spd)
  local p = self.player
  local a = math.atan2(p.y - y, p.x - x)
  if n <= 1 then
    self:eshot(x, y, math.cos(a) * spd, math.sin(a) * spd)
    return
  end
  for i = 0, n - 1 do
    local off = (i - (n - 1) / 2) * spread
    self:eshot(x, y, math.cos(a + off) * spd, math.sin(a + off) * spd)
  end
end

function World:ring(x, y, n, spd, rot)
  rot = rot or 0
  for i = 0, n - 1 do
    local a = rot + i * math.pi * 2 / n
    self:eshot(x, y, math.cos(a) * spd, math.sin(a) * spd)
  end
end

function World:dropItem(x, y, kind)
  kind = kind or "P"
  if kind == "A" then
    self.agentDropI = (self.agentDropI or 0) + 1
    kind = ({ "C", "G", "X" })[((self.agentDropI - 1) % 3) + 1]
  end
  if AGENT_PICK[kind] then
    self:spawnTokenItem(kind, x, y)
    return
  end
  self.pickups[#self.pickups + 1] = {
    x = x, y = y, vy = 18, t = 0, kind = kind, r = 8,
  }
end

function World:spawnTokenItem(tag, x, y)
  local def = AGENT_BY_ID[AGENT_PICK[tag]]
  if not def then
    return
  end
  self.tokenItems = self.tokenItems or {}
  self.tokenItems[#self.tokenItems + 1] = {
    tag = tag, id = def.id, name = def.name, img = def.img, col = def.col,
    x = x, y = y, vy = 52, t = 0, r = 18,
  }
end

function World:updateTokenItems(dt)
  local p = self.player
  for i = #(self.tokenItems or {}), 1, -1 do
    local u = self.tokenItems[i]
    u.t = u.t + dt
    u.y = u.y + u.vy * dt
    if love.math.random() < dt * 28 then
      self:spark(u.x, u.y, (love.math.random() - 0.5) * 30, -40, u.col, 0.28, 2, "glint")
    end
    if p and p.alive and dist2(u.x, u.y, p.x, p.y) < 26 * 26 then
      table.remove(self.tokenItems, i)
      self:recruitAgent({ id = u.id })
    elseif u.y > 280 then
      table.remove(self.tokenItems, i)
    end
  end
end

function World:drawTokenItems()
  for _, u in ipairs(self.tokenItems or {}) do
    local col = u.col
    local pulse = 0.5 + 0.5 * easeCos((math.sin(u.t * 6) * 0.5 + 0.5))
    local bob = math.sin(u.t * 3.4) * 3
    local y = u.y + bob
    love.graphics.setColor(col[1], col[2], col[3], 0.16 + 0.18 * pulse)
    love.graphics.rectangle("fill", u.x - 5, y - 56, 10, 62)
    love.graphics.setColor(1, 1, 1, 0.28 + 0.4 * pulse)
    love.graphics.rectangle("fill", u.x - 1, y - 60, 2, 66)
    love.graphics.setColor(col[1], col[2], col[3], 0.35 * pulse)
    love.graphics.circle("fill", u.x, y + 8, 14 + pulse * 6)
    love.graphics.setColor(1, 1, 1, 0.22 * pulse)
    love.graphics.circle("line", u.x, y, 16 + math.sin(u.t * 8) * 3)
    love.graphics.circle("line", u.x, y, 22 + pulse * 4)
    for i = 0, 5 do
      local a = u.t * 2.2 + i * (math.pi / 3)
      local len = 10 + pulse * 6
      love.graphics.setColor(col[1], col[2], col[3], 0.55 * pulse)
      love.graphics.rectangle("fill", u.x + math.cos(a) * 8 - 1, y + math.sin(a) * 8 - 1, 2, len)
    end
    local img = self.assets[u.img]
    local sc = 1.45 + pulse * 0.25
    if img then
      love.graphics.setColor(col[1], col[2], col[3], 0.55)
      love.graphics.draw(img, math.floor(u.x) + 1, math.floor(y) + 2, 0, sc, sc, 10, 10)
      love.graphics.setColor(1, 1, 1, 1)
      love.graphics.draw(img, math.floor(u.x), math.floor(y), 0, sc, sc, 10, 10)
    end
    local tw = G.textWidth(u.name, 1)
    G.print(u.name, math.floor(u.x - tw / 2), math.floor(y - 28), { col[1], col[2], col[3], 0.9 }, 1)
  end
end

function World:nearestEnemy(x, y)
  local best, bd = nil, nil
  for _, e in ipairs(self.enemies) do
    if not e.dead then
      local d = dist2(x, y, e.x, e.y)
      if not bd or d < bd then
        best, bd = e, d
      end
    end
  end
  return best
end

function World:hasAgent(id)
  return self:getAgent(id) ~= nil
end

function World:getAgent(id)
  for _, a in ipairs(self.agents) do
    if a.id == id then return a end
  end
end

function World:recruitAgent(opts)
  opts = opts or {}
  local def = opts.id and AGENT_BY_ID[opts.id]
  if not def then
    for _, d in ipairs(AGENT_DEFS) do
      if not self:hasAgent(d.id) then
        def = d
        break
      end
    end
  end
  local px, py = self.player.x, self.player.y
  if not def then
    -- All three already out: dump tokens into the hungriest one.
    local low
    for _, a in ipairs(self.agents) do
      if not low or (a.tokens or 0) < (low.tokens or 0) then
        low = a
      end
    end
    if low then
      return self:refillAgent(low)
    end
    return
  end
  local existing = self:getAgent(def.id)
  if existing then
    return self:refillAgent(existing)
  end
  self.agents[#self.agents + 1] = {
    id = def.id, name = def.name, img = def.img, rate = def.rate, col = def.col,
    tag = def.tag, tokenMax = def.tokenMax, drain = def.drain, shotCost = def.shotCost,
    orbit = def.orbit, spin = def.spin,
    x = px, y = py, fireT = 0.45,
    eatR = def.eatR * (self.agentEatMul or 1), chomp = 0.35, eaten = 0,
    pop = opts.quiet and 1 or 0,
    tokens = def.tokenMax,
    lowWarn = false,
  }
  if opts.quiet then
    return
  end
  self.agentBoost = 2.4
  self:floater(px, py - 22, def.name, def.col)
  self:floater(px, py - 34, "TOKEN IN", def.col)
  self:startRecruitFX(def.name, def.col, px, py)
  self.audio.play("oneup")
end

function World:refillAgent(ag)
  local maxT = ag.tokenMax or 14
  ag.tokens = math.min(maxT * 1.15, (ag.tokens or 0) + maxT)
  ag.lowWarn = false
  ag.pop = 0
  self.agentBoost = 1.6
  self:floater(ag.x, ag.y - 18, "TOKEN +", ag.col)
  self:startRecruitFX(ag.name, ag.col, ag.x, ag.y)
  self.audio.play("power")
end

function World:expireAgent(i)
  local ag = self.agents[i]
  if not ag then
    return
  end
  local tag = ag.tag or "C"
  local x, y = ag.x, math.max(28, ag.y - 10)
  self:floater(ag.x, ag.y - 16, "TOKEN OUT", ag.col)
  self:burst(ag.x, ag.y, 18, ag.col, 160)
  self:popRing(ag.x, ag.y, ag.col, 0.4)
  table.remove(self.agents, i)
  self:spawnTokenItem(tag, x, math.max(20, y - 28))
end

function World:countTokenPickups()
  return #(self.tokenItems or {})
end

function World:nextMissingToken()
  for _, d in ipairs(AGENT_DEFS) do
    if not self:hasAgent(d.id) then
      return d.tag
    end
  end
  return ({ "C", "G", "X" })[((self.agentDropI or 0) % 3) + 1]
end

function World:startRecruitFX(name, col, x, y)
  self.recruitFX = {
    name = name, col = col,
    x = x, y = y,
    t = 0, life = 2.2,
  }
  self.flash = 0.18
  self:burst(x, y, 28, col, 180)
  self:burst(x, y, 10, G.palette.white, 90)
  for i = 0, 11 do
    local a = i * (math.pi * 2 / 12)
    self:spark(x, y, math.cos(a) * 140, math.sin(a) * 140, col, 0.45, 2.4, "glint")
    self:spark(x, y, math.cos(a + 0.3) * 70, math.sin(a + 0.3) * 70, G.palette.white, 0.32, 1.6, "glint")
  end
  self:popRing(x, y, col, 0.55)
  self:popRing(x, y, G.palette.white, 0.32)
end

function World:nearestThreatBullet(ag, maxD)
  maxD = maxD or 80
  local p = self.player
  local best, bd = nil, nil
  for _, b in ipairs(self.ebullets) do
    if dist2(b.x, b.y, p.x, p.y) <= maxD * maxD then
      local d = dist2(b.x, b.y, ag.x, ag.y)
      if not bd or d < bd then
        best, bd = b, d
      end
    end
  end
  return best
end

function World:agentShouldShoot(ag)
  if self.agentAutoFire then
    return true
  end
  local r = (ag.eatR or 28) * 2.2
  local r2 = r * r
  for _, b in ipairs(self.ebullets) do
    if dist2(b.x, b.y, ag.x, ag.y) <= r2 then
      return false
    end
  end
  return true
end

function World:agentShoot(ag)
  local dmgMul = self.agentDmgMul or 0.4
  local auto = self.agentAutoFire
  if ag.id == "claude" then
    local t = self:nearestEnemy(ag.x, ag.y)
    local a = t and math.atan2(t.y - ag.y, t.x - ag.x) or -math.pi / 2
    local spd = auto and 280 or 260
    local dmg = (auto and 1.15 or 0.5) * dmgMul
    self:pshot(ag.x, ag.y, math.cos(a) * spd, math.sin(a) * spd, dmg, { col = ag.col, agent = true, r = 3 })
    self:spark(ag.x, ag.y, math.cos(a) * 40, math.sin(a) * 40, ag.col, 0.18, 2, "glint")
  elseif ag.id == "grok" then
    local wob = math.sin(self.time * 11) * (auto and 28 or 36)
    if auto then
      self:pshot(ag.x, ag.y, wob - 90, -240, 0.55 * dmgMul, { col = ag.col, agent = true })
      self:pshot(ag.x, ag.y, wob - 40, -265, 0.55 * dmgMul, { col = ag.col, agent = true })
      self:pshot(ag.x, ag.y, wob, -285, 0.7 * dmgMul, { col = ag.col, agent = true })
      self:pshot(ag.x, ag.y, wob + 40, -265, 0.55 * dmgMul, { col = ag.col, agent = true })
      self:pshot(ag.x, ag.y, wob + 90, -240, 0.55 * dmgMul, { col = ag.col, agent = true })
    else
      self:pshot(ag.x, ag.y, wob, -250, 0.4 * dmgMul, { col = ag.col, agent = true })
    end
  elseif ag.id == "codex" then
    if auto then
      self:pshot(ag.x - 4, ag.y, 0, -300, 0.5 * dmgMul, { col = ag.col, agent = true, r = 2 })
      self:pshot(ag.x + 4, ag.y, 0, -300, 0.5 * dmgMul, { col = ag.col, agent = true, r = 2 })
      if math.floor(self.time * 8) % 2 == 0 then
        self:pshot(ag.x, ag.y, -18, -290, 0.4 * dmgMul, { col = ag.col, agent = true, r = 2 })
        self:pshot(ag.x, ag.y, 18, -290, 0.4 * dmgMul, { col = ag.col, agent = true, r = 2 })
      end
    else
      self:pshot(ag.x, ag.y, 0, -280, 0.35 * dmgMul, { col = ag.col, agent = true, r = 2 })
    end
  end
end

function World:updateAgents(dt)
  local p = self.player
  local n = #self.agents
  local boosted = (self.agentBoost or 0) > 0
  for i = n, 1, -1 do
    local ag = self.agents[i]
    local orbit = ag.orbit or 26
    local spin = ag.spin or 2.8
    local tx, ty
    if ag.id == "codex" then
      tx = p.x
      ty = p.y - 22
    elseif ag.id == "grok" then
      local ang = self.time * spin + i * 2.2
      tx = p.x + math.cos(ang) * orbit
      ty = p.y - 18 + math.sin(ang * 1.4) * 10
    else
      local ang = self.time * spin + (i - 1) * (math.pi * 2 / math.max(1, n))
      tx = p.x + math.cos(ang) * orbit
      ty = p.y - 16 + math.sin(ang) * 8
    end
    local threat = self:nearestThreatBullet(ag, 86)
    if threat then
      tx = tx + (threat.x - tx) * 0.62
      ty = ty + (threat.y - ty) * 0.62
    end
    tx = math.max(12, math.min(180, tx))
    ty = math.max(18, math.min(236, ty))
    local follow = ag.id == "grok" and 16 or 11
    if threat then
      follow = follow + 6
    end
    ag.x = ag.x + (tx - ag.x) * math.min(1, dt * follow)
    ag.y = ag.y + (ty - ag.y) * math.min(1, dt * follow)
    ag.chomp = math.max(0, (ag.chomp or 0) - dt)
    ag.pop = (ag.pop or 1) + dt * 2.6

    local working = p.alive and self.readyT <= 0 and not self.clear and not self.over
    if working then
      ag.tokens = (ag.tokens or 0) - dt * (ag.drain or 1)
      if (ag.tokens or 0) <= (ag.tokenMax or 14) * 0.25 and not ag.lowWarn then
        ag.lowWarn = true
        self:floater(ag.x, ag.y - 14, "LOW TOKEN", ag.col)
      end
    end

    local sparkRate = ag.id == "grok" and 36 or (ag.id == "claude" and 14 or 22)
    if love.math.random() < dt * (boosted and sparkRate * 2 or sparkRate) then
      self:spark(ag.x, ag.y, (love.math.random() - 0.5) * 28, (love.math.random() - 0.5) * 28, ag.col, 0.2, 1.6, "glint")
    end
    if working then
      local fireMul = 1
      if self.agentAutoFire then
        fireMul = boosted and 1.7 or 1
      end
      ag.fireT = ag.fireT - dt * fireMul
      if ag.fireT <= 0 then
        ag.fireT = ag.rate * (self.agentRateMul or 1)
        if self:agentShouldShoot(ag) then
          self:agentShoot(ag)
          ag.tokens = (ag.tokens or 0) - (ag.shotCost or 0.1)
        else
          ag.fireT = math.min(ag.fireT, 0.28)
        end
      end
    end
    if (ag.tokens or 0) <= 0 then
      self:expireAgent(i)
    end
  end
end

function World:killEnemy(e, silent)
  if e.dead then return end
  e.dead = true
  e.hp = 0
  self.kills = self.kills + 1
  self.combo = self.combo + 1
  self.comboT = 1.4
  local mul = 1 + math.min(4, math.floor(self.combo / 8)) * 0.25
  self:addScore(e.score * mul * self.loop)
  if not silent then
    self:boom(e.x, e.y, e.boss and 2.2 or 1)
    self:popRing(e.x, e.y, G.palette.yellow, e.boss and 0.55 or 0.28)
    self:starBurst(e.x, e.y, G.palette.white)
    if e.boss then
      self.audio.play("explodeBig")
      self.shake = 10
      self.flash = 0.35
    else
      self.audio.play("explode")
      self.shake = math.max(self.shake, 2)
    end
  end
  if e.drop then
    self:dropItem(e.x, e.y, e.drop)
  elseif e.boss then
    self:dropItem(e.x, e.y, "A")
    self:dropItem(e.x - 16, e.y + 8, "B")
    self:dropItem(e.x + 16, e.y + 8, "P")
    self:dropItem(e.x, e.y + 16, "U")
  elseif e.midboss then
    self:dropItem(e.x, e.y, e.drop or "A")
    self:dropItem(e.x - 14, e.y + 8, "P")
    self:dropItem(e.x + 14, e.y + 8, "B")
    self.shake = math.max(self.shake, 6)
    self.audio.play("explodeBig")
  elseif love.math.random() < (e.kind == "spider" and 0.55 or (e.kind == "clippy" and 0.5 or (e.kind == "infloop" and 0.45 or (e.kind == "leak" and 0.45 or (e.kind == "overflow" and 0.5 or (e.kind == "deadlock" and 0.4 or (e.kind == "worm" and 0.35 or 0.1))))))) then
    local roll = love.math.random()
    local k = "P"
    if roll > 0.96 then k = "1UP"
    elseif roll > 0.62 then k = "A"
    elseif roll > 0.52 then k = "U"
    elseif roll > 0.44 then k = "N"
    elseif roll > 0.32 then k = "B"
    elseif roll > 0.20 then k = "S"
    end
    if e.kind == "overflow" and love.math.random() < 0.45 then k = "A" end
    if #(self.agents or {}) == 0 and love.math.random() < 0.35 then k = "A" end
    self:dropItem(e.x, e.y, k)
  end
  if e.boss then
    self.clear = true
    self.clearT = 3.5
    self.ebullets = {}
    self:addScore(20000 * self.loop)
  end
end

function World:doBomb()
  if self.bombs <= 0 or self.bombing > 0 then return false end
  local p = self.player
  self.bombs = self.bombs - 1
  self.bombing = 0.9
  self.bombFlash = 1
  self.shake = 8
  self.ownRing = { x = p.x, y = p.y, r = 6, t = 0, life = 0.72 }
  self.audio.play("bomb")
  self.audio.play("explodeBig")
  self:floater(p.x, p.y - 20, "OWNERSHIP", G.palette.rust)
  self:burst(p.x, p.y, 22, G.palette.rust, 140)
  self:burst(p.x, p.y, 14, G.palette.yellow, 90)
  -- Take ownership of enemy bullets: DROP some, MOVE the rest into rust shots.
  for i = #self.ebullets, 1, -1 do
    local b = self.ebullets[i]
    self:addScore(25)
    self:spark(b.x, b.y, 0, -40, G.palette.rust, 0.28, 2)
    if love.math.random() < 0.4 then
      self:pshot(b.x, b.y, 0, -240, 1.3, { col = G.palette.rust, r = 3 })
    end
    table.remove(self.ebullets, i)
  end
  for _, e in ipairs(self.enemies) do
    if not e.dead then
      e.hp = e.hp - 16
      self:burst(e.x, e.y, 8, G.palette.cyan, 90)
    end
  end
  p.inv = math.max(p.inv, 2.0)
  if p.dying then
    p.dying = false
    p.dieT = 0
    p.inv = 2.6
  end
  return true
end

function World:hurtPlayer()
  local p = self.player
  if not p.alive or p.inv > 0 or self.bombing > 0 then return end
  if p.dying then return end
  -- Death-bomb window
  p.dying = true
  p.dieT = 0.28
end

function World:reallyDie()
  local p = self.player
  p.dying = false
  p.alive = false
  self.power = math.max(1, self.power - 1)
  if #self.agents > 0 then
    local lost = table.remove(self.agents)
    self:floater(p.x, p.y - 22, lost.name .. " DOWN", lost.col)
  end
  self:boom(p.x, p.y, 1.4)
  self.audio.play("death")
  self.shake = 7
  self.lives = self.lives - 1
  if self.power >= 2 then
    self:dropItem(p.x, p.y - 10, "P")
  end
  if self.lives < 0 then
    self.over = true
    return
  end
  p.alive = true
  p.x, p.y = 96, 230
  p.inv = 3.4
  p.flash = 0
end

function World:firePlayer()
  local p = self.player
  local pow = self.power
  local y = p.y - 14
  local pierce = self.unwrapT > 0
  local extra = { pierce = pierce, trail = true }
  if self.unsafeT > 0 then
    extra.col = G.palette.lred
  elseif pierce then
    extra.col = G.palette.yellow
  else
    extra.col = ({ G.palette.cyan, G.palette.lgreen, G.palette.yellow, G.palette.cyan })[pow]
  end
  self.muzzle = 0.08
  self:spark(p.x, y + 4, 0, 40, extra.col, 0.12, 2)
  self:spark(p.x - 3, y + 6, -20, 30, G.palette.white, 0.1, 1)
  self:spark(p.x + 3, y + 6, 20, 30, G.palette.white, 0.1, 1)
  if pow == 1 then
    self:pshot(p.x, y, 0, -320, 1.1, extra)
  elseif pow == 2 then
    -- TRAIT: two owned shots
    extra.col = G.palette.lgreen
    self:pshot(p.x - 6, y + 2, 0, -330, 1.15, extra)
    self:pshot(p.x + 6, y + 2, 0, -330, 1.15, extra)
  elseif pow == 3 then
    -- ASYNC: three-way
    extra.col = G.palette.yellow
    self:pshot(p.x, y, 0, -350, 1.35, extra)
    self:pshot(p.x - 9, y + 4, -50, -315, 1.1, extra)
    self:pshot(p.x + 9, y + 4, 50, -315, 1.1, extra)
  else
    -- TOKIO: wide + homing missiles
    extra.col = G.palette.cyan
    self:pshot(p.x, y, 0, -360, 1.5, extra)
    self:pshot(p.x - 8, y + 2, -30, -340, 1.1, extra)
    self:pshot(p.x + 8, y + 2, 30, -340, 1.1, extra)
    self:fireTokio(p.x, y)
  end
  self.audio.play(pow >= 3 and "shot2" or "shot")
end

function World:fireTokio(x, y)
  local extra = { homing = true, missile = true, trail = true, r = 4, life = 1.6, col = G.palette.rust }
  self:pshot(x - 12, y + 6, -40, -220, 1.6, extra)
  self:pshot(x + 12, y + 6, 40, -220, 1.6, extra)
end

function World:updateEnemy(e, dt)
  e.t = e.t + dt
  e.flash = math.max(0, (e.flash or 0) - dt)
  e.hpFlash = math.max(0, (e.hpFlash or 0) - dt)
  e.hpShow = math.max(0, (e.hpShow or 0) - dt)
  local pth = e.path
  if pth == "down" then
    e.y = e.y + e.speed * dt
  elseif pth == "sine" then
    e.y = e.y + e.speed * dt
    e.x = e.x0 + math.sin(e.t * 3.2) * 30
  elseif pth == "zigzag" then
    e.y = e.y + e.speed * dt
    e.x = e.x + e.vx * dt
    if e.x < 16 or e.x > 176 then
      e.vx = -e.vx
      e.x = math.max(16, math.min(176, e.x))
    end
  elseif pth == "swoop" then
    e.y = e.y + e.speed * dt
    e.x = e.x0 + math.sin(e.t * 2.2) * 50
    if e.t > 1.2 and e.vy then
      e.y = e.y + 20 * dt
    end
  elseif pth == "hover" then
    if e.y < 54 then
      e.y = e.y + 46 * dt
    else
      e.x = e.x0 + math.sin(e.t * 1.3) * (e.midboss and 36 or 48)
      local leave = e.midboss and 20 or 6.5
      if e.t > leave then
        e.y = e.y + (e.midboss and 22 or 40) * dt
      end
    end
  elseif pth == "dive" then
    if e.t < 0.7 then
      e.y = e.y + e.speed * dt
    else
      local p = self.player
      local a = math.atan2(p.y - e.y, p.x - e.x)
      e.x = e.x + math.cos(a) * e.speed * 1.15 * dt
      e.y = e.y + math.sin(a) * e.speed * 1.15 * dt
    end
  elseif pth == "side" then
    local vx = (e.vx ~= 0) and e.vx or e.speed
    e.x = e.x + vx * dt
    e.y = e.y0 + 52 + math.sin(e.t * 2.1) * 22
  elseif pth == "teleport" then
    e.y = e.y + e.speed * 0.6 * dt
    e.visible = math.floor(e.t * 8) % 3 ~= 0
    if math.floor(e.t * 2) ~= math.floor((e.t - dt) * 2) and love.math.random() < 0.4 then
      e.x = 24 + love.math.random() * 144
    end
  elseif pth == "orbit" then
    e.x = e.x0 + math.cos(e.t * 2.4) * 42
    e.y = e.y0 + e.t * 22 + math.sin(e.t * 2.4) * 18
  elseif pth == "boss" then
    -- handled in updateBoss
  end

  if e.kind == "heisen" then
    e.visible = math.floor(e.t * 10) % 4 ~= 0
    e.tint = e.visible and { 0.7, 0.9, 1, 0.85 } or { 0.7, 0.9, 1, 0.15 }
  elseif e.kind == "lifetime" then
    e.visible = math.floor(e.t * 7) % 5 ~= 0
    e.tint = e.visible and { 0.65, 1, 1, 0.92 } or { 0.65, 1, 1, 0.18 }
  elseif e.kind == "infloop" then
    e.tint = { 1, 0.7 + 0.3 * math.sin(e.t * 8), 1, 1 }
  end

  if e.fire and e.fire > 0 and e.y > 8 and e.y < 190 and not e.boss then
    e.fireT = e.fireT - dt * (self.fireRateMul or 1)
    if e.fireT <= 0 then
      e.fireT = e.fire * (0.85 + love.math.random() * 0.3)
      local spd = (48 + self.loop * 5) * (self.bulletSpdMul or 1)
      if e.kind == "moth" then
        self:aimed(e.x, e.y + 8, 1, 0, spd)
      elseif e.kind == "spider" then
        self:aimed(e.x, e.y + 8, 2, 0.18, spd - 8)
      elseif e.kind == "leak" then
        self:eshot(e.x, e.y + 10, 0, spd * 0.55, 4, G.palette.lgreen)
        self:eshot(e.x - 8, e.y + 8, -20, spd * 0.5, 3, G.palette.lgreen)
        self:eshot(e.x + 8, e.y + 8, 20, spd * 0.5, 3, G.palette.lgreen)
      elseif e.kind == "overflow" then
        self:ring(e.x, e.y, e.midboss and 8 or 5, spd - 10, e.t)
        if e.midboss then
          self:aimed(e.x, e.y + 12, 1, 0, spd)
        end
      elseif e.kind == "deadlock" then
        self:eshot(e.x - 10, e.y, -spd, 18, 3, G.palette.magenta)
        self:eshot(e.x + 10, e.y, spd, 18, 3, G.palette.cyan)
        self:aimed(e.x, e.y + 6, 1, 0, spd - 12)
      elseif e.kind == "heisen" and e.visible then
        self:aimed(e.x, e.y + 6, 2, 0.26, spd + 10)
      elseif e.kind == "offby1" then
        self:aimed(e.x, e.y + 8, 2, 0.14, spd) -- off by one: one extra shot
      elseif e.kind == "clippy" then
        self:aimed(e.x, e.y + 8, 2, 0.2, spd - 12)
        self:eshot(e.x, e.y + 10, 0, spd * 0.45, 4, G.palette.yellow)
      elseif e.kind == "lifetime" and e.visible ~= false then
        self:aimed(e.x, e.y + 6, 1, 0, spd + 16)
      elseif e.kind == "infloop" then
        self:ring(e.x, e.y, 4, spd - 16, e.t * 2)
      end
    end
  end

  if e.kind == "panic" and e.y > 40 then
    if e.t > 1.4 or dist2(e.x, e.y, self.player.x, self.player.y) < 20 * 20 then
      self:ring(e.x, e.y, 6, 58, e.t)
      self:killEnemy(e)
    end
  end
end

function World:updateBoss(e, dt)
  e.pt = e.pt + dt
  e.t = e.t + dt
  e.flash = math.max(0, (e.flash or 0) - dt)
  e.hpFlash = math.max(0, (e.hpFlash or 0) - dt)
  e.hpShow = math.max(0, (e.hpShow or 0) - dt)
  if e.y < 48 then
    e.y = e.y + 26 * dt
    self.phaseName = e.bossTitle or "BOSS"
    return
  end

  local maxhp = e.maxhp or (DEFS[e.kind] and DEFS[e.kind].hp or 200)
  local u = e.hp / maxhp
  local phase
  if u > 0.7 then
    phase = 1
  elseif u > 0.42 then
    phase = 2
  else
    phase = 3
  end
  if phase ~= e.phase then
    e.phase = phase
    e.pt = 0
    local names
    if e.kind == "bossOverflow" then
      names = { "STACK WALK", "FRAME SMASH", "OVERFLOW" }
    elseif e.kind == "bossDeadlock" then
      names = { "WAIT LOCK", "JOIN HANG", "POISON PILL" }
    else
      names = { "SIGSEGV", "NULL DEREF", "KERNEL PANIC" }
    end
    self.phaseName = names[phase]
    self:floater(96, 70, names[phase], G.palette.lred)
    self.audio.play("warn")
    self.shake = 3
  end

  local sway = 28 + phase * 8
  e.x = 96 + math.sin(e.t * (0.5 + phase * 0.1)) * sway
  e.fireT = e.fireT - dt * (self.fireRateMul or 1)
  local spd = (40 + self.loop * 5 + phase * 3 + (self.stage - 1) * 3) * (self.bulletSpdMul or 1)

  if e.kind == "bossOverflow" then
    if phase == 1 and e.fireT <= 0 then
      e.fireT = 0.85
      self:aimed(e.x, e.y + 24, 2, 0.16, spd)
    elseif phase == 2 and e.fireT <= 0 then
      e.fireT = 1.05
      for i = -1, 1 do
        self:eshot(e.x + i * 16, e.y + 20, 0, spd + 6)
      end
    elseif phase == 3 and e.fireT <= 0 then
      e.fireT = 0.42
      self:aimed(e.x, e.y + 22, 1, 0, spd + 8)
      self:eshot(e.x - 20, e.y + 10, -24, spd)
      self:eshot(e.x + 20, e.y + 10, 24, spd)
    end
    self.laser = nil
    return
  end

  if e.kind == "bossDeadlock" then
    if phase == 1 and e.fireT <= 0 then
      e.fireT = 0.95
      self:ring(e.x, e.y + 8, 6, spd - 10, e.t)
    elseif phase == 2 and e.fireT <= 0 then
      e.fireT = 1.15
      self:aimed(e.x - 18, e.y + 16, 2, 0.2, spd)
      self:aimed(e.x + 18, e.y + 16, 2, 0.22, spd)
    elseif phase == 3 and e.fireT <= 0 then
      e.fireT = 0.5
      self:ring(e.x, e.y + 10, 7, spd - 4, e.t * 1.2)
      if love.math.random() < 0.2 then
        self:spawn("deadlock", love.math.random() < 0.5 and -16 or 210, 10, {
          path = "side", vx = love.math.random() < 0.5 and 40 or -50,
        })
      end
    end
    self.laser = nil
    return
  end

  -- SEGMENTATION FAULT (final)
  if phase == 1 and e.fireT <= 0 then
    e.fireT = 0.8
    self:aimed(e.x, e.y + 28, 2, 0.18, spd)
  elseif phase == 2 and e.fireT <= 0 then
    e.fireT = 1.2
    self:ring(e.x, e.y + 12, 8, spd - 8, e.t)
  elseif phase == 3 and e.fireT <= 0 then
    e.fireT = 0.28
    local a = e.t * 4.4
    self:eshot(e.x - 18, e.y + 10, math.cos(a) * spd, math.sin(a) * spd)
    self:eshot(e.x + 18, e.y + 10, math.cos(-a) * spd, math.sin(-a) * spd)
  end

  if phase >= 3 then
    if not self.laser then
      self.laser = { x = self.player.x, t = 0, state = "aim" }
    end
    local L = self.laser
    L.t = L.t + dt
    if L.state == "aim" then
      L.x = L.x + (self.player.x - L.x) * dt * 1.4
      if L.t > 1.5 then
        L.state = "fire"
        L.t = 0
        self.audio.play("warn2")
      end
    elseif L.state == "fire" then
      if self.player.alive and math.abs(self.player.x - L.x) < 7 and self.player.y > 40 then
        self:hurtPlayer()
      end
      if L.t > 0.28 then
        self.laser = nil
      end
    end
  else
    self.laser = nil
  end
end

-- Visible horizontal range in playfield coords (see src/display.lua).
-- Wider than 0..192 on wide windows; ambient stuff (stars, streaks,
-- clouds) should cover it, and bullets should not vanish inside it.
function World:viewSpan()
  local Display = package.loaded["src.display"]
  local x0 = Display and Display.viewLeft or 0
  local x1 = Display and Display.viewRight or 192
  return math.min(0, x0), math.max(192, x1)
end

function World:update(dt, input)
  do
    local vx0, vx1 = self:viewSpan()
    self.cullL, self.cullR = vx0 - 16, vx1 + 16
  end
  if self.hitstop > 0 then
    self.hitstop = self.hitstop - dt
    dt = dt * 0.15
  end
  self.shake = math.max(0, self.shake - dt * 18)
  self.bombing = math.max(0, self.bombing - dt)
  self.bombFlash = math.max(0, self.bombFlash - dt * 1.6)
  self.comboT = self.comboT - dt
  if self.comboT <= 0 then self.combo = 0 end
  self.bgY = self.bgY + 52 * dt
  self.time = self.time + dt
  if self.warningT > 0 then self.warningT = self.warningT - dt end
  if self.readyT > 0 then self.readyT = self.readyT - dt end
  self.unwrapT = math.max(0, self.unwrapT - dt)
  self.unsafeT = math.max(0, self.unsafeT - dt)
  self.tokioT = math.max(0, (self.tokioT or 0) - dt)
  self.muzzle = math.max(0, self.muzzle - dt)
  self.flash = math.max(0, (self.flash or 0) - dt * 2.8)
  self.agentBoost = math.max(0, (self.agentBoost or 0) - dt)
  if self.recruitFX then
    local fx = self.recruitFX
    fx.t = fx.t + dt
    local u = fx.t / fx.life
    if u < 0.55 then
      local spin = fx.t * 14
      for i = 0, 2 do
        local a = spin + i * 2.094
        local rad = 18 + easeCos(u / 0.55) * 42
        self:spark(
          fx.x + math.cos(a) * rad,
          fx.y + math.sin(a) * rad * 0.7,
          math.cos(a) * 30, math.sin(a) * 30,
          fx.col, 0.28, 2.2, "glint"
        )
      end
      if math.floor(fx.t * 10) ~= math.floor((fx.t - dt) * 10) then
        self:popRing(fx.x, fx.y, fx.col, 0.28)
      end
    end
    if fx.t >= fx.life then
      self.recruitFX = nil
    end
  end

  local vx0, vx1 = self:viewSpan()
  local streakCap = math.floor(18 * (vx1 - vx0) / 192)
  if #self.streaks < streakCap and love.math.random() < dt * 22 * (vx1 - vx0) / 192 then
    self.streaks[#self.streaks + 1] = {
      x = love.math.random(vx0, vx1 - 1),
      y = -8,
      len = 8 + love.math.random() * 18,
      v = 180 + love.math.random() * 220,
      a = 0.12 + love.math.random() * 0.18,
    }
  end
  for i = #self.streaks, 1, -1 do
    local s = self.streaks[i]
    s.y = s.y + s.v * dt
    if s.y > 270 then
      table.remove(self.streaks, i)
    end
  end
  for i = #(self.fx or {}), 1, -1 do
    local q = self.fx[i]
    q.t = q.t + dt
    q.r = q.r + dt * 95
    if q.t >= q.life then
      table.remove(self.fx, i)
    end
  end
  if self.ownRing then
    self.ownRing.t = self.ownRing.t + dt
    self.ownRing.r = 8 + self.ownRing.t * 180
    if self.ownRing.t >= self.ownRing.life then
      self.ownRing = nil
    end
  end

  self.propT = self.propT + dt
  local street = (self.stage or 1) == 1
  local gap = street and 1.25 or 2.8
  local cap = street and 6 or 4
  if self.propT > gap and not self.over and #self.scenery < cap then
    self.propT = 0
    if street then
      self.shopI = (self.shopI or 0) + 1
      local kind = STREET_SHOPS[((self.shopI - 1) % #STREET_SHOPS) + 1]
      local left = self.shopI % 2 == 1
      local x = left and 36 or 156
      self:spawnProp(kind, x, -32, {
        sc = 0.7 + love.math.random() * 0.2,
        vy = 34 + love.math.random() * 18,
        a = 0.95,
      })
    else
      local x = (vx0 + 24) + love.math.random() * ((vx1 - vx0) - 48)
      self:spawnCloud(x, -16, {
        sc = 0.32 + love.math.random() * 0.28,
        vy = 82 + love.math.random() * 28,
        a = 0.16 + love.math.random() * 0.16,
      })
    end
  end

  -- Shining agent objects sit on the scrolling map.
  self.tokenRainT = (self.tokenRainT or 0) + dt
  local rainGap = (#self.agents == 0) and (self.tokenRainEmpty or 6.4) or (self.tokenRainBusy or 10.5)
  if self.readyT <= 0 and not self.over and not self.clear
      and self:countTokenPickups() < 2 and self.tokenRainT >= rainGap then
    self.tokenRainT = 0
    self.agentDropI = (self.agentDropI or 0) + 1
    local lane = ({ 52, 96, 140 })[1 + (self.agentDropI - 1) % 3]
    self:spawnTokenItem(self:nextMissingToken(), lane, -24)
  end

  self:updateTokenItems(dt)

  for i = #self.scenery, 1, -1 do
    local s = self.scenery[i]
    s.y = s.y + s.vy * dt
    s.rot = s.rot + (s.vr or 0) * dt
    if s.y > 300 then
      table.remove(self.scenery, i)
    end
  end

  for i = #self.ghosts, 1, -1 do
    local g = self.ghosts[i]
    g.t = g.t - dt
    if g.t <= 0 then
      table.remove(self.ghosts, i)
    end
  end

  for _, st in ipairs(self.stars) do
    st.y = st.y + st.v * dt
    if st.y > 256 then
      st.y = st.y - 256
      local vx0, vx1 = self:viewSpan()
      st.x = love.math.random(vx0, vx1 - 1)
    end
  end

  -- Script
  if self.script and not self.clear and not self.over then
    local t = self.time
    while self.eventI <= #self.script do
      local ev = self.script[self.eventI]
      if t < ev[1] then break end
      ev[2](self)
      self.eventI = self.eventI + 1
    end
    if self.eventI > #self.script then
      self.scriptDone = true
    end
  end

  if self.clear then
    self.clearT = self.clearT - dt
    -- drain remaining bugs
    for _, e in ipairs(self.enemies) do
      if not e.dead and not e.boss then
        e.hp = e.hp - dt * 20
      end
    end
  end

  local p = self.player
  if p.dying then
    p.dieT = p.dieT - dt
    if input.bomb then
      self:doBomb()
    elseif p.dieT <= 0 then
      self:reallyDie()
    end
  end

  if p.alive and not self.over and not self.clear then
    local spd = input.shoot and 108 or 138
    local dx, dy = 0, 0
    if input.left then dx = dx - 1 end
    if input.right then dx = dx + 1 end
    if input.up then dy = dy - 1 end
    if input.down then dy = dy + 1 end
    if dx ~= 0 and dy ~= 0 then
      dx, dy = dx * 0.707, dy * 0.707
    end
    p.x = p.x + dx * spd * dt
    p.y = p.y + dy * spd * dt
    -- The ship may use the whole visible width (see src/display.lua):
    -- wider than the 192 playfield on wide windows, narrower when cropped.
    local Display = require "src.display"
    local x0 = (Display.viewLeft or 0) + 10
    local x1 = (Display.viewRight or 192) - 10
    p.x = math.max(x0, math.min(x1, p.x))
    p.y = math.max(18, math.min(242, p.y))
    p.inv = math.max(0, p.inv - dt)
    p.flash = p.flash + dt
    p.fireT = p.fireT - dt
    if math.floor(self.time * 40) ~= math.floor((self.time - dt) * 40) then
      self.ghosts[#self.ghosts + 1] = { x = p.x, y = p.y, t = 0.16 }
    end
    self:spark(p.x, p.y + 16, (love.math.random() - 0.5) * 12, 50, G.palette.cyan, 0.18, 1.2)
    if input.shoot and p.fireT <= 0 and self.readyT <= 0 then
      local interval = self.power >= 4 and 0.055 or 0.07
      if (self.agentBoost or 0) > 0 then
        interval = interval * 0.62
      end
      p.fireT = interval
      self:firePlayer()
    end
    if (self.tokioT or 0) > 0 and p.fireT > 0 then
      self.tokioFire = (self.tokioFire or 0) - dt
      if self.tokioFire <= 0 and input.shoot then
        self.tokioFire = 0.2
        self:fireTokio(p.x, p.y - 12)
      end
    end
    if input.bomb then
      self:doBomb()
    end
  end

  -- BORROWCK lane: thin beam that shreds bullets and chips bugs
  self.pbeam = nil
  if p.alive and self.power >= 4 and input.shoot and self.readyT <= 0
      and not self.clear and not self.over then
    self.pbeam = p.x
  end

  self:updateAgents(dt)

  -- Player bullets
  for i = #self.pbullets, 1, -1 do
    local b = self.pbullets[i]
    if b.homing then
      local t = self:nearestEnemy(b.x, b.y)
      if t then
        local a = math.atan2(t.y - b.y, t.x - b.x)
        local spd = math.sqrt(b.vx * b.vx + b.vy * b.vy)
        if spd < 80 then spd = 240 end
        local nx, ny = math.cos(a) * spd, math.sin(a) * spd
        b.vx = b.vx + (nx - b.vx) * math.min(1, dt * 5)
        b.vy = b.vy + (ny - b.vy) * math.min(1, dt * 5)
      end
    end
    b.x = b.x + b.vx * dt
    b.y = b.y + b.vy * dt
    b.life = b.life - dt
    if b.trail then
      self:spark(b.x, b.y + 4, -b.vx * 0.05, -b.vy * 0.08, b.col or G.palette.cyan, 0.14, b.agent and 1 or 1.6)
    end
    if b.y < -12 or b.life <= 0 or b.x < (self.cullL or -16) or b.x > (self.cullR or 208) then
      table.remove(self.pbullets, i)
    end
  end

  -- Enemies
  for i = #self.enemies, 1, -1 do
    local e = self.enemies[i]
    if e.dead then
      table.remove(self.enemies, i)
    else
      if e.boss then
        self:updateBoss(e, dt)
      else
        self:updateEnemy(e, dt)
      end
      if self.pbeam and e.visible ~= false and e.y < p.y
          and math.abs(e.x - self.pbeam) < (e.r * 0.55) then
        e.hp = e.hp - 14 * dt
        if love.math.random() < dt * 8 then
          self:burst(e.x, e.y, 1, G.palette.cyan, 36)
        end
      end
      if e.hp <= 0 then
        self:killEnemy(e)
      elseif e.y > 276 or e.y < -60 or e.x < (self.cullL or -16) - 34 or e.x > (self.cullR or 208) + 34 then
        if not e.boss then
          table.remove(self.enemies, i)
        end
      end
    end
  end

  -- Pbullets vs enemies
  for i = #self.pbullets, 1, -1 do
    local b = self.pbullets[i]
    local hit = false
    for _, e in ipairs(self.enemies) do
      if not e.dead and e.visible ~= false and dist2(b.x, b.y, e.x, e.y) < (e.r + b.r) * (e.r + b.r) then
        if not (b.hits[e.id]) then
          b.hits[e.id] = true
          e.hp = e.hp - b.dmg
          hit = true
          e.flash = 0.1
          self:burst(b.x, b.y, 3, b.col or G.palette.cyan, 50)
          self.audio.play("hit")
          if e.boss then
            e.hpFlash = 0.22
            e.hpShow = 1.8
            self.shake = math.max(self.shake, 1.8)
          end
          if e.hp <= 0 then
            self.hitstop = 0.03
            if b.agent then
              self:agentPop(e.x, e.y, b.col)
            end
          end
        end
        if not b.pierce then
          break
        end
      end
    end
    if hit and not b.pierce then
      table.remove(self.pbullets, i)
    end
  end

  -- Enemy bullets
  for i = #self.ebullets, 1, -1 do
    local b = self.ebullets[i]
    b.x = b.x + b.vx * dt
    b.y = b.y + b.vy * dt
    b.life = b.life - dt
    if b.y < -16 or b.y > 272 or b.x < (self.cullL or -16) or b.x > (self.cullR or 208) or b.life <= 0 then
      table.remove(self.ebullets, i)
    elseif self.pbeam and math.abs(b.x - self.pbeam) < 3.4 and b.y < p.y then
      table.remove(self.ebullets, i)
      self:addScore(4)
      self:burst(b.x, b.y, 1, G.palette.cyan, 30)
    elseif self.bombing > 0 then
      table.remove(self.ebullets, i)
      self:burst(b.x, b.y, 2, G.palette.yellow, 40)
    elseif #self.agents > 0 and self:tryAgentEat(b.x, b.y) then
      table.remove(self.ebullets, i)
    elseif p.alive then
      local d2 = dist2(b.x, b.y, p.x, p.y)
      if d2 < 8 * 8 and not b.grazed then
        b.grazed = true
        self.graze = self.graze + 1
        self:addScore(10)
        self.audio.play("graze")
        self:burst(p.x, p.y, 2, G.palette.white, 40)
      end
      if d2 < (2.0 + b.r) * (2.0 + b.r) then
        table.remove(self.ebullets, i)
        self:hurtPlayer()
      end
    end
  end

  -- Body collision
  if p.alive and p.inv <= 0 then
    for _, e in ipairs(self.enemies) do
      if not e.dead and e.visible ~= false and dist2(p.x, p.y, e.x, e.y) < (e.r * 0.55 + 2) ^ 2 then
        self:hurtPlayer()
        break
      end
    end
  end

  -- Pickups
  for i = #self.pickups, 1, -1 do
    local u = self.pickups[i]
    u.t = u.t + dt
    u.y = u.y + u.vy * dt
    u.vy = math.min(42, u.vy + 14 * dt)
    u.x = u.x + math.sin(u.t * 4) * 8 * dt
    if p.alive then
      local pull = (self.bombing > 0) and 8 or 3.2
      u.x = u.x + (p.x - u.x) * dt * pull
      u.y = u.y + (p.y - u.y) * dt * pull
    end
    local grab = 26
    if p.alive and dist2(u.x, u.y, p.x, p.y) < grab * grab then
      if u.kind == "P" then
        if self.power < 4 then
          self.power = self.power + 1
          self:floater(u.x, u.y, SKILL_NAME[self.power] or "POWER", G.palette.cyan)
        else
          self:addScore(1000)
          self:floater(u.x, u.y, "1000", G.palette.yellow)
        end
        self.audio.play("power")
      elseif u.kind == "B" then
        self.bombs = math.min(6, self.bombs + 1)
        self:floater(u.x, u.y, "OWN BOMB", G.palette.rust)
        self.audio.play("power")
      elseif u.kind == "S" then
        self:addScore(2000)
        self:floater(u.x, u.y, "2000", G.palette.yellow)
        self.audio.play("coin")
      elseif u.kind == "1UP" then
        self.lives = math.min(9, self.lives + 1)
        self:floater(u.x, u.y, "1UP", G.palette.yellow)
        self.audio.play("oneup")
      elseif AGENT_PICK[u.kind] then
        self:recruitAgent({ id = AGENT_PICK[u.kind] })
      elseif u.kind == "U" then
        self.unwrapT = 8
        self:floater(u.x, u.y, "UNWRAP", G.palette.yellow)
        self.audio.play("power")
      elseif u.kind == "T" then
        self.tokioT = 10
        self:floater(u.x, u.y, "TOKIO", G.palette.rust)
        self.audio.play("power")
      elseif u.kind == "N" then
        self.unsafeT = 6
        self:floater(u.x, u.y, "UNSAFE", G.palette.lred)
        self.audio.play("power")
      end
      table.remove(self.pickups, i)
    elseif u.y > 270 then
      table.remove(self.pickups, i)
    end
  end

  -- Particles / booms / floaters
  for i = #self.particles, 1, -1 do
    local q = self.particles[i]
    q.t = q.t + dt
    q.x = q.x + q.vx * dt
    q.y = q.y + q.vy * dt
    q.vy = q.vy + 40 * dt
    if q.t >= q.life then table.remove(self.particles, i) end
  end
  for i = #self.booms, 1, -1 do
    local q = self.booms[i]
    q.t = q.t + dt
    if q.t > 0.35 then table.remove(self.booms, i) end
  end
  for i = #self.floaters, 1, -1 do
    local q = self.floaters[i]
    q.t = q.t - dt
    q.y = q.y - 18 * dt
    if q.t <= 0 then table.remove(self.floaters, i) end
  end
end

function World:drawBg()
  -- Faint Raiden clouds only (under bullets/enemies). Ground map is the window fill.
  for _, s in ipairs(self.streaks) do
    love.graphics.setColor(1, 1, 1, s.a or 0.14)
    love.graphics.rectangle("fill", math.floor(s.x), math.floor(s.y), 1, s.len)
  end
  for _, s in ipairs(self.scenery) do
    local a = s.a or 0.22
    local sc = s.sc or 0.5
    local img = self.assets[s.img]
    local bob = math.sin((self.time or 0) * 3.2 + (s.x or 0) * 0.04) * 1.4
    if img then
      love.graphics.setColor(1, 1, 1, a)
      love.graphics.draw(img, s.x, s.y + bob, 0, sc, sc, s.w / 2, s.h / 2)
    else
      love.graphics.setColor(0.92, 0.95, 1, a)
      local r = 6 * sc
      love.graphics.circle("fill", s.x, s.y, r)
      love.graphics.circle("fill", s.x - r * 0.75, s.y + r * 0.12, r * 0.65)
      love.graphics.circle("fill", s.x + r * 0.7, s.y + r * 0.08, r * 0.55)
    end
  end
end

function World:draw()
  self:drawBg()
  self:drawTokenItems()

  -- Pickups
  for _, u in ipairs(self.pickups) do
    love.graphics.setColor(1, 1, 1, 1)
    love.graphics.draw(self.assets.pickup, u.x, u.y, u.t * 2, 1, 1, 9, 9)
    local col = G.palette.yellow
    if u.kind == "P" then col = G.palette.cyan
    elseif u.kind == "B" then col = G.palette.rust
    elseif u.kind == "1UP" then col = G.palette.lgreen
    elseif u.kind == "C" then col = { 0.95, 0.84, 0.18 }
    elseif u.kind == "G" then col = { 0.95, 0.22, 0.22 }
    elseif u.kind == "X" then col = { 0.22, 0.88, 0.34 }
    elseif u.kind == "U" then col = G.palette.yellow
    elseif u.kind == "T" then col = G.palette.rust
    elseif u.kind == "N" then col = G.palette.lred
    end
    local label = u.kind == "1UP" and "1" or u.kind
    G.print(label, u.x - G.textWidth(label, 1) / 2, u.y - 3, col, 1)
  end

  -- Enemies
  for _, e in ipairs(self.enemies) do
    if not e.dead then
      local img = self.assets[e.img] or self.assets.beetle
      local t = e.tint
      love.graphics.setColor(t[1], t[2], t[3], t[4])
      local ox, oy = e.w / 2, e.h / 2
      local rot = 0
      if e.kind == "moth" or e.kind == "heisen" then rot = math.sin(e.t * 8) * 0.15 end
      if e.kind == "worm" then rot = math.sin(e.t * 5) * 0.2 end
      if e.kind == "leak" then rot = math.sin(e.t * 2) * 0.08 end
      if e.kind == "infloop" then rot = e.t * 5 end
      if e.kind == "clippy" then rot = math.sin(e.t * 3) * 0.25 end
      if e.kind == "offby1" then rot = 0.12 end
      love.graphics.setColor(0, 0, 0, 0.35)
      love.graphics.draw(img, math.floor(e.x) + 1, math.floor(e.y) + 2, rot, 1, 1, ox, oy)
      if (e.flash or 0) > 0 then
        love.graphics.setColor(1, 1, 1, 1)
      else
        love.graphics.setColor(t[1], t[2], t[3], t[4])
      end
      love.graphics.draw(img, math.floor(e.x), math.floor(e.y), rot, 1, 1, ox, oy)
      if (e.boss or e.midboss) and (e.maxhp or 0) > 0 then
        local bw = e.boss and 56 or 36
        local bh = e.boss and 5 or 3
        local bx = math.floor(e.x - bw / 2)
        local by = math.floor(e.y - oy - 8)
        local u = math.max(0, math.min(1, e.hp / e.maxhp))
        love.graphics.setColor(0, 0, 0, 0.8)
        love.graphics.rectangle("fill", bx - 1, by - 1, bw + 2, bh + 2)
        love.graphics.setColor(G.palette.dred)
        love.graphics.rectangle("fill", bx, by, bw * u, bh)
        if (e.hpFlash or 0) > 0 then
          love.graphics.setColor(1, 1, 1, 0.75)
          love.graphics.rectangle("fill", bx, by, bw * u, bh)
        else
          love.graphics.setColor(G.palette.yellow)
          love.graphics.rectangle("fill", bx, by, bw * u, 2)
        end
      end
    end
  end

  -- BORROWCK player beam
  if self.pbeam then
    local x = self.pbeam
    local py = self.player.y - 16
    love.graphics.setColor(0.35, 0.95, 1, 0.22 + 0.08 * math.sin(self.time * 28))
    love.graphics.rectangle("fill", x - 3, 0, 6, py)
    love.graphics.setColor(0.85, 1, 1, 0.85)
    love.graphics.rectangle("fill", x - 1, 0, 2, py)
  end

  -- Crown laser
  if self.laser then
    local L = self.laser
    if L.state == "aim" then
      love.graphics.setColor(1, 0.85, 0.3, 0.35 + 0.25 * math.sin(self.time * 20))
      love.graphics.rectangle("fill", L.x - 5, 0, 10, 256)
    else
      love.graphics.setColor(1, 0.3, 0.2, 0.75)
      love.graphics.rectangle("fill", L.x - 8, 0, 16, 256)
      love.graphics.setColor(1, 1, 0.8, 0.95)
      love.graphics.rectangle("fill", L.x - 2, 0, 4, 256)
    end
  end

  -- Player bullets
  for _, b in ipairs(self.pbullets) do
    if b.missile and self.assets.tokio then
      local ang = math.atan2(b.vy, b.vx) + math.pi / 2
      love.graphics.setColor(1, 1, 1, 1)
      love.graphics.draw(self.assets.tokio, b.x, b.y, ang, 1, 1, 6, 9)
    else
      if b.col then
        love.graphics.setColor(b.col[1], b.col[2], b.col[3], 0.55)
        love.graphics.rectangle("fill", math.floor(b.x) - 3, math.floor(b.y) - 1, 6, 13)
        love.graphics.setColor(b.col[1], b.col[2], b.col[3], 1)
      elseif b.pierce then
        love.graphics.setColor(1, 0.95, 0.4, 1)
      else
        love.graphics.setColor(1, 1, 1, 1)
      end
      love.graphics.draw(self.assets.pbullet, b.x, b.y, 0, b.pierce and 1.25 or 1.12, b.pierce and 1.4 or 1.12, 5, 7)
      love.graphics.setColor(1, 1, 1, 1)
      love.graphics.rectangle("fill", math.floor(b.x) - 1, math.floor(b.y) - 3, 3, 7)
    end
  end

  -- Ship afterimages + AI agents
  for _, g in ipairs(self.ghosts) do
    local a = math.max(0, g.t / 0.16) * 0.35
    love.graphics.setColor(0.4, 0.9, 1, a)
    love.graphics.draw(self.assets.player, math.floor(g.x), math.floor(g.y), 0, 1, 1, 20, 20)
  end
  for _, ag in ipairs(self.agents) do
    local img = self.assets[ag.img]
    if img then
      local pop = ag.pop or 1
      local sc
      local rot = 0
      if pop < 1 then
        sc = 0.18 + easeCos(pop) * 1.22
        rot = (1 - easeCos(pop)) * 6.4
      else
        local settle = math.max(0, 1.35 - pop)
        sc = 1 + math.sin((pop - 1) * 14) * 0.1 * settle
      end
      if (ag.chomp or 0) > 0 then
        sc = sc * (1.12 + ag.chomp * 0.8)
      end
      local tokenU = math.max(0, math.min(1, (ag.tokens or 0) / (ag.tokenMax or 14)))
      local flicker = 1
      if tokenU < 0.25 then
        flicker = (math.floor(self.time * 14) % 2 == 0) and 1 or 0.35
      end
      local aura = 7 + math.sin(self.time * (ag.id == "grok" and 16 or 8)) * 2
      if (self.agentBoost or 0) > 0 then
        aura = aura + 4
      end
      love.graphics.setColor(ag.col[1], ag.col[2], ag.col[3], (0.28 + tokenU * 0.25) * flicker)
      love.graphics.circle("fill", ag.x, ag.y, aura)
      love.graphics.setColor(ag.col[1], ag.col[2], ag.col[3], flicker)
      love.graphics.draw(img, math.floor(ag.x), math.floor(ag.y), rot, sc, sc, 10, 10)
      love.graphics.setColor(0, 0, 0, 0.65 * flicker)
      love.graphics.rectangle("fill", ag.x - 7, ag.y + 10, 14, 3)
      love.graphics.setColor(ag.col[1], ag.col[2], ag.col[3], 0.95 * flicker)
      love.graphics.rectangle("fill", ag.x - 7, ag.y + 10, 14 * tokenU, 3)
    end
  end

  -- Player
  local p = self.player
  if p.alive then
    local show = not (p.inv > 0 and math.floor(p.flash * 16) % 2 == 0)
    if show then
      if self.unsafeT > 0 then
        love.graphics.setColor(1, 0.55, 0.45, 1)
      elseif self.unwrapT > 0 then
        love.graphics.setColor(1, 1, 0.72, 1)
      elseif (self.agentBoost or 0) > 0 then
        local pulse = 0.55 + 0.45 * easeCos((math.sin(self.time * 8) * 0.5 + 0.5))
        love.graphics.setColor(0.75 + 0.25 * pulse, 0.95, 1, 1)
      else
        love.graphics.setColor(1, 1, 1, 1)
      end
      love.graphics.draw(self.assets.player, math.floor(p.x), math.floor(p.y), 0, 1, 1, 20, 20)
      if (self.agentBoost or 0) > 0 then
        local pulse = 0.2 + 0.25 * easeCos((math.sin(self.time * 12) * 0.5 + 0.5))
        love.graphics.setColor(0.45, 0.95, 1, pulse)
        love.graphics.circle("line", p.x, p.y, 16 + math.sin(self.time * 9) * 3)
        love.graphics.setColor(1, 1, 1, pulse * 0.7)
        love.graphics.circle("line", p.x, p.y, 11)
      end
      if self.muzzle > 0 then
        local a = self.muzzle / 0.08
        love.graphics.setColor(1, 1, 0.7, a)
        love.graphics.rectangle("fill", p.x - 3, p.y - 22, 6, 8)
        love.graphics.setColor(1, 0.55, 0.25, a)
        love.graphics.rectangle("fill", p.x - 1, p.y - 26, 2, 6)
      end
      -- Exhaust
      love.graphics.setColor(G.palette.cyan)
      local fl = 4 + math.floor(math.sin(self.time * 30) * 2)
      love.graphics.rectangle("fill", p.x - 1, p.y + 16, 2, fl)
      love.graphics.setColor(G.palette.rust)
      love.graphics.rectangle("fill", p.x - 1, p.y + 16, 2, math.max(1, fl - 2))
    end
    -- Hitbox
    love.graphics.setColor(1, 1, 1, 0.95)
    love.graphics.rectangle("fill", math.floor(p.x) - 1, math.floor(p.y) - 1, 3, 3)
    love.graphics.setColor(G.palette.cyan)
    love.graphics.rectangle("fill", math.floor(p.x), math.floor(p.y), 1, 1)
  end

  -- Enemy bullets (danmaku balls)
  for _, b in ipairs(self.ebullets) do
    local c = b.col or G.palette.magenta
    love.graphics.setColor(c)
    love.graphics.rectangle("fill", math.floor(b.x) - 3, math.floor(b.y) - 3, 6, 6)
    love.graphics.setColor(G.palette.white)
    love.graphics.rectangle("fill", math.floor(b.x) - 1, math.floor(b.y) - 1, 3, 3)
    love.graphics.setColor(c[1], c[2] * 0.5, c[3] * 0.5)
    love.graphics.rectangle("fill", math.floor(b.x) - 4, math.floor(b.y), 1, 1)
    love.graphics.rectangle("fill", math.floor(b.x) + 3, math.floor(b.y), 1, 1)
  end

  -- Explosions
  for _, q in ipairs(self.booms) do
    local sc = q.scale * (0.6 + q.t * 2.2)
    local a = 1 - q.t / 0.35
    love.graphics.setColor(1, 1, 1, a)
    love.graphics.draw(self.assets.boom, q.x, q.y, q.t * 4, sc, sc, 24, 24)
  end

  for _, q in ipairs(self.fx or {}) do
    local a = 1 - q.t / q.life
    a = easeCos(a)
    local c = q.col or G.palette.cyan
    love.graphics.setColor(c[1], c[2], c[3], 0.85 * a)
    love.graphics.circle("line", q.x, q.y, q.r)
    love.graphics.setColor(1, 1, 1, 0.45 * a)
    love.graphics.circle("line", q.x, q.y, q.r * 0.62)
  end

  for _, q in ipairs(self.particles) do
    local u = math.max(0, 1 - q.t / q.life)
    local a = easeCos(u)
    local c = q.col
    love.graphics.setColor(c[1], c[2], c[3], a)
    if q.kind == "glint" then
      local s = math.max(1, q.r * (0.45 + a))
      love.graphics.rectangle("fill", math.floor(q.x) - s, math.floor(q.y), s * 2 + 1, 1)
      love.graphics.rectangle("fill", math.floor(q.x), math.floor(q.y) - s, 1, s * 2 + 1)
      love.graphics.setColor(1, 1, 1, a)
      love.graphics.rectangle("fill", math.floor(q.x), math.floor(q.y), 1, 1)
    else
      love.graphics.rectangle("fill", math.floor(q.x), math.floor(q.y), math.max(1, q.r), math.max(1, q.r))
    end
  end

  for _, q in ipairs(self.floaters) do
    G.print(q.text, math.floor(q.x - G.textWidth(q.text, 1) / 2), math.floor(q.y), q.col, 1)
  end

  if self.bombFlash > 0 then
    love.graphics.setColor(1, 0.55, 0.25, self.bombFlash * 0.32)
    local fx0, fw = G.viewSpan()
    love.graphics.rectangle("fill", fx0, 0, fw, 256)
  end
  if self.ownRing then
    local R = self.ownRing
    local a = 1 - R.t / R.life
    love.graphics.setColor(0.86, 0.40, 0.27, 0.55 * a)
    love.graphics.circle("line", R.x, R.y, R.r)
    love.graphics.setColor(1.0, 0.85, 0.35, 0.8 * a)
    love.graphics.circle("line", R.x, R.y, R.r * 0.72)
    love.graphics.setColor(0.4, 0.9, 1, 0.35 * a)
    love.graphics.circle("line", R.x, R.y, R.r * 1.15)
  end

  -- Score / lives HUD is drawn in window space (see main.drawPlayHud)
  -- so it stays small and visible when the tall screen crops the 3:4 canvas.

  local Display = require "src.display"
  local vx = math.floor(Display.viewLeft or 0)
  local vw = math.max(80, math.floor((Display.viewRight or 192) - vx))

  local midboss
  for _, e in ipairs(self.enemies) do
    if e.midboss and not e.dead then midboss = e break end
  end
  local kingUp = self.boss and not self.boss.dead
  if self.unwrapT > 0 then
    G.print("UNWRAP", vx + 2, 20, G.palette.yellow, 1)
  elseif (self.tokioT or 0) > 0 then
    G.print("TOKIO", vx + 2, 20, G.palette.rust, 1)
  elseif self.unsafeT > 0 then
    G.print("UNSAFE", vx + 2, 20, G.palette.lred, 1)
  elseif (self.agentBoost or 0) > 0 then
    G.print("AI BOOST", vx + 2, 20, G.palette.cyan, 1)
  end

  local Stage = require "src.stage"
  if self.readyT > 0 and not self.over then
    G.center("READY", 104, G.palette.yellow, 1)
    G.center("STAGE " .. tostring(self.stage or 1), 118, G.palette.cyan, 1)
    G.center(Stage.NAMES[self.stage or 1] or "STAGE", 132, G.palette.white, 1)
    G.center("Z SHOT   X BOMB", 148, G.palette.cyan, 1)
  end

  if self.warningT > 0 then
    local flash = math.floor(self.time * 8) % 2 == 0
    local w = self.bossWarn or { "WARNING", "BOSS", "INCOMING" }
    G.center(w[1] or "WARNING", 100, flash and G.palette.lred or G.palette.yellow, 1)
    G.center(w[2] or "BOSS", 114, G.palette.white, 1)
    G.center(w[3] or "", 128, flash and G.palette.yellow or G.palette.lred, 1)
  end

  if self.clear and self.clearT > 0 then
    G.center("STAGE CLEAR", 104, G.palette.yellow, 1)
    local nxt = (self.stage or 1) < 3 and (Stage.NAMES[(self.stage or 1) + 1] or "NEXT") or "PRINCESS PITCH"
    G.center(nxt, 118, G.palette.cyan, 1)
    G.center("BONUS 20000", 132, G.palette.white, 1)
  end

  if self.combo >= 8 and not kingUp then
    G.print("COMBO " .. tostring(self.combo), vx + 2, 30, G.palette.yellow, 1)
  end

  local barX, barW = vx + 8, vw - 16
  -- Midboss HP
  if not kingUp and midboss then
    local u = math.max(0, midboss.hp / (midboss.maxhp or midboss.hp))
    love.graphics.setColor(G.palette.black)
    love.graphics.rectangle("fill", barX, 16, barW, 6)
    love.graphics.setColor(G.palette.rust)
    love.graphics.rectangle("fill", barX, 16, barW * u, 6)
    love.graphics.setColor(G.palette.yellow)
    love.graphics.rectangle("fill", barX, 16, barW * u, 2)
    G.print("OVERFLOW", barX, 24, G.palette.yellow, 1)
    local mhp = tostring(math.ceil(midboss.hp)) .. "/" .. tostring(math.ceil(midboss.maxhp or midboss.hp))
    G.print(mhp, barX + barW - G.textWidth(mhp, 1), 24, G.palette.white, 1)
  end

  -- Boss HP
  if self.boss and not self.boss.dead then
    local maxhp = math.max(1, self.boss.maxhp or self.boss.hp)
    local hp = math.max(0, self.boss.hp)
    local u = math.max(0, hp / maxhp)
    love.graphics.setColor(G.palette.black)
    love.graphics.rectangle("fill", barX, 16, barW, 8)
    if (self.boss.hpFlash or 0) > 0 then
      love.graphics.setColor(1, 1, 1, 0.9)
    else
      love.graphics.setColor(G.palette.dred)
    end
    love.graphics.rectangle("fill", barX, 16, barW * u, 8)
    love.graphics.setColor(G.palette.yellow)
    love.graphics.rectangle("fill", barX, 16, barW * u, 2)
    local title = (self.boss.bossTitle or "BOSS"):sub(1, 12)
    G.print(title, barX, 26, G.palette.lred, 1)
    local hpTxt = tostring(math.ceil(hp)) .. "/" .. tostring(math.ceil(maxhp))
    G.print(hpTxt, barX + barW - G.textWidth(hpTxt, 1), 26, G.palette.white, 1)
    if self.phaseName ~= "" then
      G.print(self.phaseName, barX, 36, G.palette.yellow, 1)
    end
  end

  if self.recruitFX then
    local fx = self.recruitFX
    local t = fx.t / fx.life
    local fade
    if t < 0.2 then
      fade = easeCos(t / 0.2)
    elseif t > 0.58 then
      fade = 1 - easeCos((t - 0.58) / 0.42)
    else
      fade = 1
    end
    local rise = math.floor((1 - easeCos(math.min(1, t / 0.25))) * -10)
    local col = fx.col or G.palette.yellow
    love.graphics.setColor(col[1], col[2], col[3], 0.28 * fade)
    love.graphics.rectangle("fill", fx.x - 6, 0, 12, 256)
    love.graphics.setColor(1, 1, 1, 0.18 * fade)
    love.graphics.rectangle("fill", fx.x - 1, 0, 2, 256)
    love.graphics.setColor(0, 0, 0, 0.55 * fade)
    local bx0, bw = G.viewSpan()
    love.graphics.rectangle("fill", bx0, 86 + rise, bw, 38)
    local name = fx.name or "AGENT"
    local scale = (#name <= 6) and 2 or 1
    local w = G.textWidth(name, scale)
    local x = math.floor(96 - w / 2)
    G.print(name, x + 1, 93 + rise, { 0, 0, 0, fade }, scale)
    G.print(name, x, 92 + rise, { fx.col[1], fx.col[2], fx.col[3], fade }, scale)
    local sub = "ONLINE"
    local sw = G.textWidth(sub, 1)
    local sx = math.floor(96 - sw / 2)
    G.print(sub, sx + 1, 115 + rise, { 0, 0, 0, fade }, 1)
    G.print(sub, sx, 114 + rise, { 1, 1, 1, fade * 0.9 }, 1)
  end
end

return World
