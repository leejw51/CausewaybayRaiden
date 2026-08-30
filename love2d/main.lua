-- CAUSEWAYBAY RAIDEN — Hong Kong vertical shmup
-- A rust programmer vs. the bugs.

local Display = require "src.display"
local G = require "src.gfx"
local Audio = require "src.audio"
local World = require "src.world"
local Stage = require "src.stage"
local Save = require "src.save"

local assets
local world
local state = "boot"
local stateT = 0
local credits = 1
local continueN = 9
local blink = 0
local hiscore = 50000
local demo = false
local bombEdge = false
local attractBugs = {}
local titleScroll = 0
local storyPage = 1
local storyPos = 0
local storyFrom = 0
local storyTo = 0
local storyAnim = 1
local storyAuto = true
local storyHold = 0
local STORY_DWELL = 3.4
local STORY_SLIDE = 0.7
local mapCursor = 1
local mapPx, mapPy = 36, 168
local verify = false
local verifyStep = 0

local MAP = {
  { x = 36, y = 176, stage = 1, tag = "1-1", title = "CAUSEWAYBAY", boss = "OVERFLOW" },
  { x = 96, y = 124, stage = 2, tag = "1-2", title = "MTR LINE", boss = "DEADLOCK" },
  { x = 156, y = 72, stage = 3, tag = "1-3", title = "HKU CASTLE", boss = "SEGFAULT" },
}

local STORY = {
  {
    img = "storyApt",
    lines = {
      "CAUSEWAYBAY 3AM",
      "TINY RUST APT",
      "ONE BUG WONT DIE",
      "NEED A BURGER",
    },
  },
  {
    img = "storyMtr",
    lines = {
      "TAKE THE MTR",
      "BUGS ON THE TRAIN",
      "DEBUG ON THE WAY",
      "NEXT STOP HKU",
    },
  },
  {
    img = "storyHku",
    lines = {
      "HKU BURGER SHOP",
      "ALMOST THERE",
      "CLEAR THE BUGS",
      "THEN WE EAT",
    },
  },
}

local function loadHiscore()
  progress = Save.load()
  hiscore = math.max(hiscore, progress.hiscore or 50000)
  mapCursor = progress.cursor or 1
  local n = MAP[mapCursor]
  if n then
    mapPx, mapPy = n.x, n.y
  end
  if love.filesystem.getInfo("hiscore.txt") then
    local old = tonumber(love.filesystem.read("hiscore.txt")) or 0
    if old > hiscore then
      hiscore = old
      Save.hiscore(hiscore)
    end
  end
end

local function saveHiscore()
  if world and world.hiscore then
    hiscore = math.max(hiscore, world.hiscore)
  end
  Save.hiscore(hiscore)
  love.filesystem.write("hiscore.txt", tostring(hiscore))
end

local function keyDown(k)
  return love.keyboard.isDown(k)
end

local function readMove()
  return {
    left = keyDown("left") or keyDown("a"),
    right = keyDown("right") or keyDown("d"),
    up = keyDown("up") or keyDown("w"),
    down = keyDown("down") or keyDown("s"),
    shoot = keyDown("z") or keyDown("space") or keyDown("j"),
    bomb = bombEdge,
  }
end

local function startGame(stage)
  demo = false
  stage = stage or (MAP[mapCursor] and MAP[mapCursor].stage) or 1
  world = World.new(assets, Audio, hiscore, stage)
  world.script = Stage.build(world.stage, 1)
  state = "play"
  stateT = 0
  Save.play(world.stage)
  Audio.play("start")
  Audio.music("stage")
end

local function easeCos(t)
  t = math.min(1, math.max(0, t))
  return (1 - math.cos(t * math.pi)) * 0.5
end

local function storyGoto(page, keepAuto)
  page = math.max(1, math.min(#STORY, page))
  if page == storyPage and storyAnim >= 1 then
    return
  end
  storyFrom = storyPos
  storyTo = page - 1
  storyPage = page
  storyAnim = 0
  storyHold = 0
  if not keepAuto then
    storyAuto = false
  end
  Audio.play("select")
end

local function startStory()
  storyPage = 1
  storyPos, storyFrom, storyTo = 0, 0, 0
  storyAnim = 1
  storyAuto = true
  storyHold = 0
  state = "story"
  stateT = 0
  Audio.play("select")
end

local function startMap()
  state = "map"
  stateT = 0
  local n = MAP[mapCursor] or MAP[1]
  mapPx, mapPy = n.x, n.y
  Audio.play("select")
  Audio.music("title")
end

local function startDemo()
  demo = true
  world = World.new(assets, Audio, hiscore)
  world.script = Stage.build(1, 1)
  world.lives = 99
  world.readyT = 0
  state = "play"
  stateT = 0
  Audio.music("stage")
end

local function nextStage()
  if world then
    Save.clear(world.stage, world.score)
    progress.cleared[world.stage] = true
    hiscore = math.max(hiscore, world.hiscore or hiscore)
  end
  if (world.stage or 1) >= 3 then
    saveHiscore()
    state = "ending"
    stateT = 0
    Audio.music("title")
  else
    mapCursor = math.min(3, (world.stage or 1) + 1)
    Save.map(mapCursor)
    startMap()
  end
end

function love.load(args)
  Display.init()
  assets = G.loadAll()
  Audio.init()
  for _, a in ipairs(args or {}) do
    if a == "--test" then
      local ok = require("src.test").run(assets, Audio, Display)
      os.exit(ok and 0 or 1)
    elseif a == "--verify" then
      verify = true
      state = "title"
      stateT = 4
    end
  end
  loadHiscore()
  math.randomseed(os.time())
  for i = 1, 8 do
    attractBugs[i] = {
      x = 20 + i * 20,
      y = -20 - i * 30,
      sp = 30 + i * 8,
    }
  end
  Audio.music("title")
end

function love.resize()
  -- integer scale is recomputed every frame
end

function love.mousepressed(x, y, button)
  if button ~= 1 then
    return
  end
  local id = Display.hitButton(x, y)
  if id == "layout" then
    Display.toggleLayout()
    Audio.play("select")
    return
  elseif id == "full" then
    Display.toggleFullscreen()
    Audio.play("blip")
    return
  end
  if state == "story" then
    local sw = love.graphics.getDimensions()
    if x < sw * 0.5 then
      if storyPage > 1 then
        storyGoto(storyPage - 1, false)
      else
        storyAuto = false
      end
    else
      if storyPage < #STORY then
        storyGoto(storyPage + 1, false)
      else
        storyAuto = false
        startMap()
      end
    end
  end
end

local function systemChord()
  return love.keyboard.isDown("lgui", "rgui", "lctrl", "rctrl")
end

function love.keypressed(k)
  -- Let macOS keep Shift+Command+5 (and other OS chords) for screen recording.
  if systemChord() then
    return
  end
  if k == "f" or k == "f11" then
    Display.toggleFullscreen()
    Audio.play("blip")
    return
  end
  if k == "tab" or k == "l" then
    Display.toggleLayout()
    Audio.play("select")
    return
  end
  if k == "c" then
    credits = math.min(9, credits + 1)
    Audio.play("coin")
    return
  end

  if state == "title" or state == "boot" then
    if k == "return" or k == "kpenter" or k == "z" or k == "space" then
      if credits > 0 then
        startStory()
      else
        Audio.play("blip")
      end
    end
  elseif state == "story" then
    if k == "left" or k == "a" then
      if storyPage > 1 then
        storyGoto(storyPage - 1, false)
      else
        storyAuto = false
      end
    elseif k == "right" or k == "d" then
      if storyPage < #STORY then
        storyGoto(storyPage + 1, false)
      else
        storyAuto = false
        startMap()
      end
    elseif k == "space" or k == "return" or k == "kpenter" or k == "z" then
      startMap()
    end
  elseif state == "map" then
    if k == "left" or k == "a" then
      mapCursor = mapCursor <= 1 and 3 or (mapCursor - 1)
      Save.map(mapCursor)
      Audio.play("select")
    elseif k == "right" or k == "d" then
      mapCursor = mapCursor >= 3 and 1 or (mapCursor + 1)
      Save.map(mapCursor)
      Audio.play("select")
    elseif k == "return" or k == "kpenter" or k == "z" or k == "space" then
      if credits > 0 then
        credits = math.max(0, credits - 1)
      end
      startGame(MAP[mapCursor].stage)
    elseif k == "escape" then
      state = "title"
      stateT = 0
      Audio.music("title")
    end
  elseif state == "ending" then
    if k == "return" or k == "z" or k == "space" or k == "escape" then
      state = "title"
      stateT = 0
      Audio.music("title")
    end
  elseif state == "play" then
    if k == "p" or k == "escape" then
      if not demo then
        state = "pause"
        Audio.play("select")
      else
        state = "title"
        stateT = 0
        Audio.music("title")
      end
    end
    if k == "x" or k == "lshift" or k == "rshift" or k == "k" then
      bombEdge = true
    end
    if demo and (k == "return" or k == "z" or k == "space") then
      if credits > 0 then
        credits = credits - 1
        startGame()
      else
        state = "title"
        stateT = 0
        Audio.music("title")
      end
    end
  elseif state == "pause" then
    if k == "p" or k == "escape" or k == "return" then
      state = "play"
      Audio.play("select")
    end
  elseif state == "continue" then
    if k == "return" or k == "z" or k == "space" then
      if credits > 0 then
        credits = credits - 1
        world.over = false
        world.lives = 5
        world.bombs = 5
        world.player.alive = true
        world.player.x, world.player.y = 96, 220
        world.player.inv = 2.5
        world.player.dying = false
        state = "play"
        Audio.play("start")
        Audio.music(world.boss and not world.boss.dead and "boss" or "stage")
      end
    elseif k == "escape" then
      saveHiscore()
      startMap()
    end
  elseif state == "gameover" then
    if k == "return" or k == "z" or k == "space" or k == "escape" then
      startMap()
    end
  end
end

function love.update(dt)
  if dt > 0.05 then dt = 0.05 end
  stateT = stateT + dt
  blink = blink + dt

  if state == "boot" then
    titleScroll = titleScroll + 10 * dt
    if stateT > 6.2 then
      state = "title"
      stateT = 0
    end
    return
  end

  if state == "story" then
    titleScroll = titleScroll + 12 * dt
    if storyAnim < 1 then
      storyAnim = math.min(1, storyAnim + dt / STORY_SLIDE)
      storyPos = storyFrom + (storyTo - storyFrom) * easeCos(storyAnim)
    else
      storyPos = storyTo
      if storyAuto then
        storyHold = storyHold + dt
        if storyHold >= STORY_DWELL then
          storyHold = 0
          if storyPage < #STORY then
            storyGoto(storyPage + 1, true)
          else
            startMap()
          end
        end
      end
    end
    return
  end

  if state == "map" then
    local n = MAP[mapCursor] or MAP[1]
    mapPx = mapPx + (n.x - mapPx) * math.min(1, dt * 8)
    mapPy = mapPy + (n.y - mapPy) * math.min(1, dt * 8)
    return
  end

  if state == "ending" then
    if stateT > 16 then
      state = "title"
      stateT = 0
    end
    return
  end

  if state == "title" then
    titleScroll = titleScroll + 18 * dt
    for _, b in ipairs(attractBugs) do
      b.y = b.y + b.sp * dt
      -- Stay in the hero art, not over the title text.
      if b.y > 96 then
        b.y = -24
        b.x = 28 + love.math.random() * 136
      end
    end
    if stateT > 18 then
      startDemo()
    end
    return
  end

  if state == "pause" then
    return
  end

  if state == "continue" then
    continueN = continueN - dt
    if continueN <= 0 then
      saveHiscore()
      state = "gameover"
      stateT = 0
      Audio.music("over")
    end
    return
  end

  if state == "gameover" then
    if stateT > 8 then
      startMap()
    end
    return
  end

  if state == "play" and world then
    local input
    if demo then
      local t = world.time
      input = {
        left = math.sin(t * 1.3) < -0.3,
        right = math.sin(t * 1.3) > 0.3,
        up = math.sin(t * 0.7) > 0.4,
        down = math.sin(t * 0.7) < -0.2,
        shoot = true,
        bomb = false,
      }
    else
      input = readMove()
    end
    world:update(dt, input)
    bombEdge = false

    if world.clear and world.clearT <= 0 then
      nextStage()
    end
    if world.over and not demo then
      saveHiscore()
      state = "continue"
      continueN = 9.9
      Audio.stopMusic()
      Audio.play("death")
    elseif world.over and demo then
      state = "title"
      stateT = 0
      Audio.music("title")
    end
    if demo and stateT > 25 then
      state = "title"
      stateT = 0
      Audio.music("title")
    end
  end
end

local function colA(col, a)
  return { col[1], col[2], col[3], a }
end

local function bootFade(t, t0, t1)
  local fade = 0.85
  if t < t0 or t > t1 then
    return 0
  end
  local u = t - t0
  local dur = t1 - t0
  if u < fade then
    return easeCos(u / fade)
  end
  if u > dur - fade then
    return easeCos((dur - u) / fade)
  end
  return 1
end

local function drawBoot()
  love.graphics.clear(0, 0, 0, 0)
  local t = stateT
  local a1 = bootFade(t, 0.05, 2.55)
  local a2 = bootFade(t, 2.5, 6.15)
  if a1 > 0.02 then
    G.center("HONG KONG", 96, colA(G.palette.cyan, a1), 1)
    G.center("NIGHT 2026", 118, colA(G.palette.yellow, a1), 1)
  end
  if a2 > 0.02 then
    G.center("CAUSEWAYBAY", 88, colA(G.palette.rust, a2), 1)
    G.center("AI", 110, colA(G.palette.yellow, a2), 2)
    G.center("PRESENTS", 144, colA(G.palette.cyan, a2), 1)
    G.center("CAUSEWAYBAY", 168, colA(G.palette.white, a2), 1)
    G.center("RAIDEN", 182, colA(G.palette.white, a2), 1)
  end
end

local function drawTitle()
  love.graphics.clear(0, 0, 0, 0)

  -- Bugs behind the art/text so they never cover the menu.
  for _, b in ipairs(attractBugs) do
    love.graphics.setColor(1, 1, 1, 0.9)
    love.graphics.draw(assets.beetle, b.x, b.y, 0, 1, 1, 13, 13)
  end

  love.graphics.setColor(1, 1, 1, 1)
  love.graphics.draw(assets.title, 0, 0)

  love.graphics.setColor(0.03, 0.02, 0.10, 0.78)
  love.graphics.rectangle("fill", 0, 104, 192, 152)

  local flash = math.floor(blink * 8) % 2 == 0
  G.center("CAUSEWAYBAY", 108, flash and G.palette.yellow or G.palette.rust, 1)
  G.center("RAIDEN", 122, G.palette.cyan, 2)
  G.center(credits > 0 and "SPACE START" or "C  INSERT COIN", 150, flash and G.palette.yellow or G.palette.white, 1)
  G.center("Z SHOT   X BOMB", 166, G.palette.cyan, 1)
  G.center("ARROWS MOVE", 180, G.palette.white, 1)
  G.center("P PAUSE  F FULL", 194, G.palette.magenta, 1)
end

local function drawMap()
  love.graphics.clear(0, 0, 0, 0)
  if assets.worldMap then
    love.graphics.setColor(1, 1, 1, 1)
    love.graphics.draw(assets.worldMap, 0, 0)
  end
  love.graphics.setColor(0.03, 0.02, 0.10, 0.55)
  love.graphics.rectangle("fill", 0, 0, 192, 28)
  love.graphics.rectangle("fill", 0, 214, 192, 42)
  G.center("WORLD 1 HONG KONG", 8, G.palette.yellow, 1)

  -- Dotted path between nodes
  for i = 1, #MAP - 1 do
    local a, b = MAP[i], MAP[i + 1]
    local steps = 8
    for s = 1, steps do
      local u = s / (steps + 1)
      local x = a.x + (b.x - a.x) * u
      local y = a.y + (b.y - a.y) * u
      love.graphics.setColor(1, 0.85, 0.35, 0.9)
      love.graphics.rectangle("fill", math.floor(x) - 1, math.floor(y) - 1, 2, 2)
    end
  end

  for i, n in ipairs(MAP) do
    local on = i == mapCursor
    local cleared = progress.cleared[n.stage]
    love.graphics.setColor(0, 0, 0, 0.55)
    love.graphics.circle("fill", n.x + 1, n.y + 2, on and 10 or 7)
    if cleared then
      love.graphics.setColor(0.92, 0.82, 0.35)
    elseif on then
      love.graphics.setColor(G.palette.cyan)
    else
      love.graphics.setColor(G.palette.white)
    end
    love.graphics.circle("fill", n.x, n.y, on and 9 or 6)
    love.graphics.setColor(G.palette.navy)
    love.graphics.circle("line", n.x, n.y, on and 9 or 6)
    G.print(n.tag, n.x - 12, n.y - 18, on and G.palette.yellow or G.palette.white, 1)
    if cleared then
      love.graphics.push()
      love.graphics.translate(n.x + 2, n.y + 1)
      love.graphics.rotate(-0.38)
      love.graphics.push()
      love.graphics.scale(1.65, 0.72)
      love.graphics.setColor(0.82, 0.14, 0.16, 0.28)
      love.graphics.circle("fill", 0, 0, 10)
      love.graphics.setColor(0.82, 0.14, 0.16, 0.95)
      love.graphics.setLineWidth(2)
      love.graphics.circle("line", 0, 0, 10)
      love.graphics.pop()
      love.graphics.pop()
      G.print("CLR", n.x - 10, n.y - 3, { 0.92, 0.18, 0.16, 1 }, 1)
      love.graphics.setLineWidth(1)
    end
  end

  -- Walker
  love.graphics.setColor(1, 1, 1, 1)
  local bob = math.sin(blink * 8) * 1.5
  if assets.player then
    love.graphics.draw(assets.player, math.floor(mapPx), math.floor(mapPy + bob) - 8, 0, 0.55, 0.55, 20, 20)
  end

  local n = MAP[mapCursor] or MAP[1]
  G.center(n.tag .. "  " .. n.title, 218, G.palette.cyan, 1)
  G.center("BOSS " .. n.boss, 232, G.palette.rust, 1)
  G.center(progress.cleared[n.stage] and "CLEARED" or "SPACE PLAY", 244, G.palette.yellow, 1)
end

local function drawStoryPage(page, ox)
  if math.abs(ox) >= 192 then
    return
  end
  love.graphics.push()
  love.graphics.translate(ox, 0)
  local img = assets[page.img]
  if img then
    love.graphics.setColor(1, 1, 1, 1)
    love.graphics.draw(img, 0, 0)
  end
  love.graphics.setColor(0.03, 0.02, 0.10, 0.78)
  local x0 = G.viewLeft or 0
  local vw = (G.viewRight or 192) - x0
  love.graphics.rectangle("fill", x0, 138, vw, 118)
  local y = 158
  for _, line in ipairs(page.lines) do
    G.center(line, y, G.palette.white, 1)
    y = y + 14
  end
  love.graphics.pop()
end

local function drawStory()
  love.graphics.clear(0, 0, 0, 0)
  love.graphics.setScissor(0, 0, 192, 256)
  for i, page in ipairs(STORY) do
    drawStoryPage(page, (i - 1 - storyPos) * 192)
  end
  love.graphics.setScissor()
  G.center("STORY " .. tostring(storyPage) .. "/" .. tostring(#STORY), 142, G.palette.cyan, 1)
end

local function drawEnding()
  love.graphics.clear(0, 0, 0, 0)
  local endArt = assets.storyEnd or assets.storyHku
  if endArt then
    love.graphics.setColor(1, 1, 1, 1)
    love.graphics.draw(endArt, 0, 0)
  end
  love.graphics.setColor(0.03, 0.02, 0.10, 0.7)
  love.graphics.rectangle("fill", 0, 132, 192, 124)
  G.center("THE END", 138, G.palette.yellow, 1)
  G.center("PRINCESS PITCH", 154, G.palette.magenta, 1)
  G.center("HKU BURGER CLEAR", 168, G.palette.cyan, 1)
  if assets.princess then
    love.graphics.setColor(1, 1, 1, 1)
    love.graphics.draw(assets.princess, 96, 210, 0, 1, 1, 16, 16)
  end
  if world then
    G.center(string.format("%06d", world.score), 186, G.palette.white, 1)
  end
end

local function drawPause()
  love.graphics.setColor(0, 0, 0, 0.6)
  love.graphics.rectangle("fill", 0, 0, 192, 256)
  G.center("PAUSE", 72, G.palette.yellow, 2)
  G.center("Z SHOT", 112, G.palette.cyan, 1)
  G.center("X BOMB", 126, G.palette.rust, 1)
  G.center("C  COIN", 140, G.palette.yellow, 1)
  G.center("ARROWS MOVE", 154, G.palette.white, 1)
  G.center("P ESC RESUME", 176, G.palette.white, 1)
  G.center("F FULLSCREEN", 190, G.palette.magenta, 1)
end

local function drawContinue()
  love.graphics.setColor(0, 0, 0, 0.6)
  love.graphics.rectangle("fill", 0, 0, 192, 256)
  G.center("CONTINUE?", 90, G.palette.yellow, 2)
  G.center(tostring(math.max(0, math.ceil(continueN))), 118, G.palette.lred, 3)
  G.center("CREDIT " .. tostring(credits), 160, G.palette.cyan, 1)
  if credits > 0 then
    G.center("SPACE START", 178, G.palette.white, 1)
  else
    G.center("C  INSERT COIN", 178, G.palette.gray, 1)
  end
end

local function drawGameover()
  love.graphics.setColor(0, 0, 0, 0.65)
  love.graphics.rectangle("fill", 0, 0, 192, 256)
  G.center("GAME OVER", 96, G.palette.lred, 2)
  if world then
    G.center("SCORE " .. string.format("%06d", world.score), 128, G.palette.yellow, 1)
    G.center("GRAZE " .. tostring(world.graze), 144, G.palette.cyan, 1)
    G.center("KILLS " .. tostring(world.kills), 156, G.palette.white, 1)
    G.center("AGENTS " .. tostring(#(world.agents or {})), 168, G.palette.magenta, 1)
  end
  G.center("THE BUGS WON", 188, G.palette.gray, 1)
end

local function drawBezels(x, y, s, sw, sh)
  -- Leftover space is already filled with stage background.
  -- Horizontal mode only overlays HUD on the sides.
  if Display.layout ~= "horizontal" then
    return
  end
  local leftW = math.max(0, x)
  local rightX = x + Display.GW * s
  local rightW = math.max(0, sw - rightX)

  local fs = math.max(1, math.min(2, math.floor(s * 0.4)))
  local function tprint(str, sx, sy, col)
    G.print(str, sx, sy, col, fs)
  end

  if leftW > 40 then
    local lx = math.floor(leftW * 0.08)
    tprint("1UP", lx, y + 16, G.palette.cyan)
    tprint(world and string.format("%06d", world.score) or "000000", lx, y + 16 + 10 * fs, G.palette.white)
    tprint("LIVES", lx, y + 16 + 28 * fs, G.palette.rust)
    tprint(world and tostring(math.max(0, world.lives)) or "3", lx, y + 16 + 38 * fs, G.palette.yellow)
    tprint("BOMB", lx, y + 16 + 54 * fs, G.palette.rust)
    tprint(world and tostring(world.bombs) or "3", lx, y + 16 + 64 * fs, G.palette.yellow)
    tprint("AGENT", lx, y + 16 + 80 * fs, G.palette.cyan)
    tprint(world and tostring(#world.agents) or "0", lx, y + 16 + 90 * fs, G.palette.white)
    tprint("VIEW", lx, sh - 48 * fs, G.palette.gray)
    tprint("HORZ", lx, sh - 36 * fs, G.palette.yellow)
  end
  if rightW > 40 then
    local rx = rightX + math.floor(rightW * 0.12)
    tprint("HI", rx, y + 16, G.palette.cyan)
    tprint(string.format("%06d", world and world.hiscore or hiscore), rx, y + 16 + 10 * fs, G.palette.yellow)
    tprint("POWER", rx, y + 16 + 28 * fs, G.palette.cyan)
    tprint(world and tostring(world.power) or "1", rx, y + 16 + 38 * fs, G.palette.white)
    tprint("LOOP", rx, y + 16 + 54 * fs, G.palette.magenta)
    tprint(world and tostring(world.loop) or "1", rx, y + 16 + 64 * fs, G.palette.white)
    local skills = { "PRINTLN", "TRAIT", "ASYNC", "TOKIO" }
    tprint("SKILL", rx, y + 16 + 80 * fs, G.palette.cyan)
    tprint(world and (skills[world.power] or "PRINTLN") or "PRINTLN", rx, y + 16 + 90 * fs, G.palette.yellow)
    tprint("TAB", rx, sh - 48 * fs, G.palette.gray)
    tprint("HORZ", rx, sh - 36 * fs, G.palette.yellow)
  end
end

function love.draw()
  Display.begin()

  if state == "boot" then
    drawBoot()
  elseif state == "title" then
    drawTitle()
  elseif state == "story" then
    drawStory()
  elseif state == "map" then
    drawMap()
  elseif state == "ending" then
    drawEnding()
  elseif state == "play" or state == "pause" or state == "continue" or state == "gameover" then
    if world then
      world:draw()
    end
    if demo and state == "play" then
      if math.floor(blink * 2) % 2 == 0 then
        G.center("DEMO", 88, G.palette.yellow, 1)
        G.center("SPACE START", 200, G.palette.white, 1)
      end
    end
    if state == "pause" then drawPause() end
    if state == "continue" then drawContinue() end
    if state == "gameover" then drawGameover() end
  end

  local shake = (world and state ~= "title" and state ~= "boot" and state ~= "story" and state ~= "ending" and state ~= "map") and world.shake or 0
  local scroll = titleScroll
  if world and state ~= "title" and state ~= "boot" and state ~= "story" and state ~= "ending" and state ~= "map" then
    scroll = world.bgY or 0
  end
  local map = assets.bgApt or assets.bg
  local far = assets.bgAptFar or map
  local st = world and world.stage or 1
  if state == "story" then
    st = storyPage
  elseif state == "map" then
    st = mapCursor
  elseif state == "ending" then
    st = 3
  end
  if st == 2 then
    map = assets.bgMtr or map
    far = assets.bgMtrFar or map
  elseif st == 3 then
    map = assets.bgHku or map
    far = assets.bgHkuFar or map
  end
  local hazeA, cloudA = 0.34, 0.48
  if st == 1 and (state == "play" or state == "pause" or state == "continue") then
    hazeA, cloudA = 0.14, 0.14
  end
  local x, y, s, sw, sh = Display.finish(shake, {
    far = far,
    city = map,
    haze = assets.haze,
    clouds = assets.clouds,
    scroll = scroll,
    hazeA = hazeA,
    cloudA = cloudA,
  })
  drawBezels(x, y, s, sw, sh)
  drawPlayHud(sw, sh)
  drawTitleHud(sw, sh)
  drawChromeButtons(sw, sh)

  if verify then
    verifyStep = verifyStep + 1
    if verifyStep == 5 then
      love.graphics.captureScreenshot("verify_title.png")
    elseif verifyStep == 6 then
      startStory()
    elseif verifyStep == 11 then
      love.graphics.captureScreenshot("verify_story.png")
    elseif verifyStep == 12 then
      startMap()
    elseif verifyStep == 17 then
      love.graphics.captureScreenshot("verify_map.png")
    elseif verifyStep == 18 then
      startGame(1)
    elseif verifyStep == 48 then
      love.graphics.captureScreenshot("verify_play.png")
    elseif verifyStep == 52 then
      os.exit(0)
    end
  end
end

local SKILLS = { "PRINTLN", "TRAIT", "ASYNC", "TOKIO" }

local function hudScale(sw)
  return math.max(5, math.floor(sw / 155))
end

local function wcenter(str, y, col, fs, sw)
  G.printShadow(str, math.floor((sw - G.textWidth(str, fs)) / 2), y, col, fs)
end

function drawTitleHud(sw, sh)
  if state ~= "title" and state ~= "story" and state ~= "ending" and state ~= "map" then
    return
  end
  local startMsg, lines
  if state == "story" then
    startMsg = "SPACE FOR MAP"
    lines = { startMsg, "LEFT RIGHT PAGE" }
  elseif state == "map" then
    startMsg = "SPACE PLAY"
    lines = { startMsg, "LEFT RIGHT MOVE", "ESC TITLE" }
  elseif state == "ending" then
    startMsg = "SPACE TITLE"
    lines = { startMsg, "PRINCESS PITCH", "HKU BURGER CLEAR", "HONG KONG 2026" }
  else
    lines = {}
  end
  local longest = 0
  for _, s in ipairs(lines) do
    longest = math.max(longest, #s)
  end
  local fs = math.max(6, math.min(10, math.floor(sw / 110)))
  fs = math.min(fs, math.max(5, math.floor((sw - 40) / (8 * math.max(1, longest)))))
  local pad = 16
  local btn = Display.uiButtons and Display.uiButtons[1]
  local top = (btn and (btn.y + btn.h) or 36) + 10
  local hi = string.format("%06d", hiscore)
  G.printShadow(hi, pad, top, G.palette.yellow, fs)
  local cred = "CREDIT " .. tostring(credits)
  G.printShadow(cred, sw - pad - G.textWidth(cred, fs), top, G.palette.cyan, fs)

  local line = 11 * fs
  local y = sh - line * #lines - 24
  local flash = math.floor(blink * 4) % 2 == 0
  local cols = { flash and G.palette.yellow or G.palette.white, G.palette.white, G.palette.cyan, G.palette.magenta }
  for i, msg in ipairs(lines) do
    wcenter(msg, y + line * (i - 1), cols[i] or G.palette.white, fs, sw)
  end
end

function drawPlayHud(sw, sh)
  if not world then
    return
  end
  if state ~= "play" and state ~= "pause" and state ~= "continue" and state ~= "gameover" then
    return
  end
  local fs = hudScale(sw)
  local pad = 12
  local btn = Display.uiButtons and Display.uiButtons[1]
  local top = (btn and (btn.y + btn.h) or 28) + 10
  G.print(string.format("%06d", world.score), pad, top, G.palette.white, fs)
  local hi = string.format("%06d", world.hiscore)
  G.print(hi, sw - pad - G.textWidth(hi, fs), top, G.palette.yellow, fs)
  local sk = SKILLS[world.power] or "PRINTLN"
  G.print(sk, pad, top + 9 * fs, G.palette.cyan, fs)

  local bot = sh - 10 * fs - 10
  local lifeSc = fs * 0.12
  for i = 1, math.min(5, math.max(0, world.lives)) do
    love.graphics.setColor(1, 1, 1, 1)
    if assets.player then
      love.graphics.draw(assets.player, pad + 12 + (i - 1) * (fs * 5), bot + 6, 0, lifeSc, lifeSc, 20, 20)
    end
  end
  local pLabel = "P"
  local pW = G.textWidth(pLabel, fs)
  local pipW, pipH, pipGap = 6 * fs, 5 * fs, 4
  local meterW = pW + 8 + 4 * (pipW + pipGap)
  local mx = math.floor((sw - meterW) / 2)
  G.print(pLabel, mx, bot, G.palette.cyan, fs)
  for i = 1, 4 do
    local on = i <= world.power
    love.graphics.setColor(on and G.palette.cyan or G.palette.navy)
    love.graphics.rectangle("fill", mx + pW + 8 + (i - 1) * (pipW + pipGap), bot + 2, pipW, pipH)
  end
  for i = 1, world.bombs do
    love.graphics.setColor(G.palette.rust)
    love.graphics.rectangle("fill", sw - pad - i * (pipW + 6), bot + 2, pipW, pipW)
    love.graphics.setColor(G.palette.yellow)
    love.graphics.rectangle("fill", sw - pad - i * (pipW + 6) + 4, bot + 6, fs * 2, fs * 2)
  end
end

function drawChromeButtons(sw, sh)
  local mx, my = love.mouse.getPosition()
  for _, b in ipairs(Display.uiButtons or {}) do
    local hover = mx >= b.x and mx < b.x + b.w and my >= b.y and my < b.y + b.h
    local label
    if b.id == "layout" then
      label = Display.layout == "vertical" and "VERT" or "HORZ"
    else
      label = Display.fullscreen and "FULL" or "WIN"
    end
    love.graphics.setColor(hover and 0.28 or 0.10, hover and 0.16 or 0.08, hover and 0.42 or 0.22, 0.94)
    love.graphics.rectangle("fill", b.x, b.y, b.w, b.h)
    love.graphics.setColor(0.78, 0.32, 0.22, 1)
    love.graphics.rectangle("line", b.x, b.y, b.w, b.h)
    love.graphics.setColor(1.0, 0.85, 0.35, hover and 0.95 or 0.55)
    love.graphics.rectangle("line", b.x + 1, b.y + 1, b.w - 2, b.h - 2)
    local fs = b.fs or 2
    local col = hover and G.palette.yellow or G.palette.cyan
    local tw = G.textWidth(label, fs)
    G.print(label, math.floor(b.x + (b.w - tw) / 2), b.y + math.floor((b.h - 8 * fs) / 2), col, fs)
  end
end
