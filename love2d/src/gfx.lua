-- MSX1 palette, chroma-key sprite loader, 8x8 arcade font.

local G = {}

-- Authentic-ish MSX1 colors (plus rust for our hero).
G.palette = {
  black   = {0.00, 0.00, 0.00},
  navy    = {0.10, 0.10, 0.35},
  dblue   = {0.35, 0.33, 0.88},
  lblue   = {0.50, 0.46, 0.95},
  cyan    = {0.40, 0.86, 0.94},
  dgreen  = {0.23, 0.64, 0.25},
  green   = {0.24, 0.72, 0.29},
  lgreen  = {0.45, 0.82, 0.49},
  dred    = {0.73, 0.37, 0.32},
  rust    = {0.86, 0.40, 0.27},
  lred    = {1.00, 0.54, 0.49},
  dyellow = {0.80, 0.76, 0.37},
  yellow  = {0.87, 0.82, 0.53},
  magenta = {0.72, 0.40, 0.71},
  gray    = {0.80, 0.80, 0.80},
  white   = {1.00, 1.00, 1.00},
}

G.chroma = nil
G.imgs = {}

function G.init()
  G.chroma = love.graphics.newShader([[
    vec4 effect(vec4 color, Image tex, vec2 uv, vec2 sc) {
      vec4 c = Texel(tex, uv);
      // Hot magenta / pink studio backdrop -> transparent
      if (c.g < 0.46 && c.r > 0.70 && c.b > 0.70) {
        float hot = (c.r + c.b) * 0.5 - c.g;
        if (hot > 0.42) {
          c.a = 0.0;
        }
      }
      return c * color;
    }
  ]])
end

local function tightChroma(data)
  local w, h = data:getDimensions()
  for y = 0, h - 1 do
    for x = 0, w - 1 do
      local r, g, b, a = data:getPixel(x, y)
      if a > 0 and g < 0.42 and r > 0.75 and b > 0.75 then
        data:setPixel(x, y, 0, 0, 0, 0)
      end
    end
  end
end

function G.makeSprite(path, tw, th)
  local ok, img = pcall(love.graphics.newImage, path)
  if not ok then
    return G.placeholder(tw, th, G.palette.rust)
  end
  img:setFilter("linear", "linear")
  local c = love.graphics.newCanvas(tw, th)
  c:setFilter("nearest", "nearest")
  love.graphics.push("all")
  love.graphics.setCanvas(c)
  love.graphics.clear(0, 0, 0, 0)
  love.graphics.setShader(G.chroma)
  love.graphics.setColor(1, 1, 1, 1)
  love.graphics.draw(img, 0, 0, 0, tw / img:getWidth(), th / img:getHeight())
  love.graphics.setShader()
  love.graphics.setCanvas()
  love.graphics.pop()
  local data = c:newImageData()
  tightChroma(data)
  local out = love.graphics.newImage(data)
  out:setFilter("nearest", "nearest")
  return out
end

function G.makeBg(path, tw, th)
  local ok, img = pcall(love.graphics.newImage, path)
  if not ok then
    return G.placeholder(tw, th, G.palette.navy)
  end
  img:setFilter("linear", "linear")
  local c = love.graphics.newCanvas(tw, th)
  c:setFilter("nearest", "nearest")
  love.graphics.push("all")
  love.graphics.setCanvas(c)
  love.graphics.clear(0, 0, 0, 1)
  love.graphics.setColor(1, 1, 1, 1)
  love.graphics.draw(img, 0, 0, 0, tw / img:getWidth(), th / img:getHeight())
  love.graphics.setCanvas()
  love.graphics.pop()
  local out = love.graphics.newImage(c:newImageData())
  out:setFilter("nearest", "nearest")
  out:setWrap("repeat", "repeat")
  return out
end

function G.makeLayer(path, tw, th)
  local ok, img = pcall(love.graphics.newImage, path)
  if not ok then
    return nil
  end
  img:setFilter("linear", "linear")
  local c = love.graphics.newCanvas(tw, th)
  c:setFilter("nearest", "nearest")
  love.graphics.push("all")
  love.graphics.setCanvas(c)
  love.graphics.clear(0, 0, 0, 0)
  if G.chroma then
    love.graphics.setShader(G.chroma)
  end
  love.graphics.setColor(1, 1, 1, 1)
  love.graphics.draw(img, 0, 0, 0, tw / img:getWidth(), th / img:getHeight())
  love.graphics.setShader()
  love.graphics.setCanvas()
  love.graphics.pop()
  local data = c:newImageData()
  tightChroma(data)
  local out = love.graphics.newImage(data)
  out:setFilter("nearest", "nearest")
  out:setWrap("repeat", "repeat")
  return out
end

function G.placeholder(w, h, col)
  local data = love.image.newImageData(w, h)
  for y = 0, h - 1 do
    for x = 0, w - 1 do
      local edge = x == 0 or y == 0 or x == w - 1 or y == h - 1
      if edge then
        data:setPixel(x, y, 1, 1, 1, 1)
      else
        data:setPixel(x, y, col[1], col[2], col[3], 1)
      end
    end
  end
  local img = love.graphics.newImage(data)
  img:setFilter("nearest", "nearest")
  return img
end

function G.loadAll()
  G.init()
  local A = {}
  A.player   = G.makeSprite("assets/player.png", 40, 40)
  A.beetle   = G.makeSprite("assets/enemy_beetle.png", 26, 26)
  A.moth     = G.makeSprite("assets/enemy_moth.png", 32, 28)
  A.spider   = G.makeSprite("assets/enemy_spider.png", 30, 30)
  A.worm     = G.makeSprite("assets/enemy_worm.png", 28, 32)
  A.nullptr  = G.makeSprite("assets/enemy_null.png", 24, 24)
  A.leak     = G.makeSprite("assets/enemy_leak.png", 28, 28)
  A.overflow = G.makeSprite("assets/enemy_overflow.png", 30, 32)
  A.deadlock = G.makeSprite("assets/enemy_deadlock.png", 32, 24)
  A.offby1   = G.makeSprite("assets/enemy_offby1.png", 28, 26)
  A.clippy   = G.makeSprite("assets/enemy_clippy.png", 30, 30)
  A.lifetime = G.makeSprite("assets/enemy_lifetime.png", 28, 32)
  A.infloop  = G.makeSprite("assets/enemy_loop.png", 32, 32)
  A.panic    = G.makeSprite("assets/enemy_panic.png", 28, 28)
  A.boss     = G.makeSprite("assets/boss.png", 96, 88)
  A.king     = G.makeSprite("assets/king.png", 104, 92)
  A.bossOverflow = G.makeSprite("assets/boss_overflow.png", 88, 88)
  A.bossDeadlock = G.makeSprite("assets/boss_deadlock.png", 90, 80)
  A.princess = G.makeSprite("assets/princess_pitch.png", 32, 32)
  A.tokio    = G.makeSprite("assets/tokio_missile.png", 12, 18)
  A.claude   = G.makeSprite("assets/agent_claude.png", 22, 20)
  A.codex    = G.makeSprite("assets/agent_codex.png", 22, 20)
  A.grok     = G.makeSprite("assets/agent_grok.png", 22, 20)
  A.gemini   = G.makeSprite("assets/agent_gemini.png", 20, 20)
  A.pickup   = G.makeSprite("assets/pickup.png", 18, 18)
  A.boom     = G.makeSprite("assets/explosion.png", 48, 48)
  A.pbullet  = G.makeSprite("assets/bullet_player.png", 10, 14)
  A.bg       = G.makeBg("assets/bg_city.png", 192, 256)
  A.bgFar    = G.makeBg("assets/bg_far.png", 192, 256)
  A.bgMid    = G.makeBg("assets/bg_mid.png", 192, 256)
  A.bgNear   = G.makeBg("assets/bg_near.png", 192, 256)
  A.bgApt    = G.makeBg("assets/bg_apt.png", 192, 256)
  A.bgMtr    = G.makeBg("assets/bg_mtr.png", 192, 256)
  A.bgHku    = G.makeBg("assets/bg_hku.png", 192, 256)
  A.bgAptFar = G.makeBg("assets/bg_apt_far.png", 192, 256)
  A.bgMtrFar = G.makeBg("assets/bg_mtr_far.png", 192, 256)
  A.bgHkuFar = G.makeBg("assets/bg_hku_far.png", 192, 256)
  A.haze     = G.makeLayer("assets/parallax_haze.png", 192, 256)
  A.clouds   = G.makeLayer("assets/parallax_clouds.png", 192, 256)
  A.cloud    = G.makeSprite("assets/cloud.png", 48, 22)
  A.storyApt = G.makeBg("assets/story_apt.png", 192, 144)
  A.storyMtr = G.makeBg("assets/story_mtr.png", 192, 144)
  A.storyHku = G.makeBg("assets/story_hku.png", 192, 144)
  A.storyEnd = G.makeBg("assets/story_end.png", 192, 144)
  A.shopHysan  = G.makeSprite("assets/shop_hysan.png", 52, 52)
  A.shopMarket = G.makeSprite("assets/shop_market.png", 48, 48)
  A.shopRamen  = G.makeSprite("assets/shop_ramen.png", 48, 48)
  A.shopDimsum = G.makeSprite("assets/shop_dimsum.png", 48, 48)
  A.shopBakery = G.makeSprite("assets/shop_bakery.png", 48, 48)
  A.shopCoffee = G.makeSprite("assets/shop_coffee.png", 46, 46)
  A.shopCase   = G.makeSprite("assets/shop_case.png", 46, 46)
  A.title    = G.makeBg("assets/title_hero.png", 192, 144)
  A.worldMap = G.makeBg("assets/world_map.png", 192, 256)
  A.bezel    = G.makeBg("assets/bezel.png", 80, 256)
  G.imgs = A
  love.graphics.setCanvas()
  love.graphics.setShader()
  love.graphics.setColor(1, 1, 1, 1)
  collectgarbage("collect")
  return A
end

-- 5x7 glyphs packed in 8x8 cells. 1 = pixel.
local GLYPH = {}

local function pack(rows)
  local t = {}
  for i = 1, 8 do
    local row = rows[i] or "........"
    local bits = 0
    for c = 1, 8 do
      local ch = row:sub(c, c)
      if ch == "#" then
        bits = bits + 2 ^ (8 - c)
      end
    end
    t[i] = bits
  end
  return t
end

GLYPH[" "] = pack({"........","........","........","........","........","........","........","........"})
GLYPH["0"] = pack({".####...","#....#..","#...##..","#..#.#..","#.#..#..","##...#..",".####...","........"})
GLYPH["1"] = pack({"..#.....","###.....","..#.....","..#.....","..#.....","..#.....","#####...","........"})
GLYPH["2"] = pack({".####...","#....#..",".....#..","..###...","##......","#.......","######..","........"})
GLYPH["3"] = pack({".####...","#....#..",".....#..","..###...",".....#..","#....#..",".####...","........"})
GLYPH["4"] = pack({"...##...","..#.#...","..#.#...",".#..#...","######..","....#...","....#...","........"})
GLYPH["5"] = pack({"######..","#.......","#####...",".....#..",".....#..","#....#..",".####...","........"})
GLYPH["6"] = pack({".####...","#.......","#####...","#....#..","#....#..","#....#..",".####...","........"})
GLYPH["7"] = pack({"######..",".....#..","....#...","...#....","..#.....","..#.....","..#.....","........"})
GLYPH["8"] = pack({".####...","#....#..","#....#..",".####...","#....#..","#....#..",".####...","........"})
GLYPH["9"] = pack({".####...","#....#..","#....#..",".#####..",".....#..","#....#..",".####...","........"})
GLYPH["A"] = pack({"..##....",".#..#...","#....#..","######..","#....#..","#....#..","#....#..","........"})
GLYPH["B"] = pack({"#####...","#....#..","#....#..","#####...","#....#..","#....#..","#####...","........"})
GLYPH["C"] = pack({".####...","#....#..","#.......","#.......","#.......","#....#..",".####...","........"})
GLYPH["D"] = pack({"#####...","#....#..","#....#..","#....#..","#....#..","#....#..","#####...","........"})
GLYPH["E"] = pack({"######..","#.......","#.......","#####...","#.......","#.......","######..","........"})
GLYPH["F"] = pack({"######..","#.......","#.......","#####...","#.......","#.......","#.......","........"})
GLYPH["G"] = pack({".####...","#....#..","#.......","#..###..","#....#..","#....#..",".####...","........"})
GLYPH["H"] = pack({"#....#..","#....#..","#....#..","######..","#....#..","#....#..","#....#..","........"})
GLYPH["I"] = pack({"#####...","..#.....","..#.....","..#.....","..#.....","..#.....","#####...","........"})
GLYPH["J"] = pack({"..####..","....#...","....#...","....#...","....#...","#...#...",".###....","........"})
GLYPH["K"] = pack({"#...#...","#..#....","#.#.....","##......","#.#.....","#..#....","#...#...","........"})
GLYPH["L"] = pack({"#.......","#.......","#.......","#.......","#.......","#.......","######..","........"})
GLYPH["M"] = pack({"#....#..","##..##..","#.##.#..","#....#..","#....#..","#....#..","#....#..","........"})
GLYPH["N"] = pack({"#....#..","##...#..","#.#..#..","#..#.#..","#...##..","#....#..","#....#..","........"})
GLYPH["O"] = pack({".####...","#....#..","#....#..","#....#..","#....#..","#....#..",".####...","........"})
GLYPH["P"] = pack({"#####...","#....#..","#....#..","#####...","#.......","#.......","#.......","........"})
GLYPH["Q"] = pack({".####...","#....#..","#....#..","#....#..","#..#.#..","#...#...",".###.#..","........"})
GLYPH["R"] = pack({"#####...","#....#..","#....#..","#####...","#..#....","#...#...","#....#..","........"})
GLYPH["S"] = pack({".####...","#....#..","#.......",".####...",".....#..","#....#..",".####...","........"})
GLYPH["T"] = pack({"#######.","..#.....","..#.....","..#.....","..#.....","..#.....","..#.....","........"})
GLYPH["U"] = pack({"#....#..","#....#..","#....#..","#....#..","#....#..","#....#..",".####...","........"})
GLYPH["V"] = pack({"#....#..","#....#..","#....#..","#....#..",".#..#...",".#..#....","..##....","........"})
GLYPH["W"] = pack({"#....#..","#....#..","#....#..","#....#..","#.##.#..","##..##..","#....#..","........"})
GLYPH["X"] = pack({"#....#..",".#..#...","..##....","..##....","..##....",".#..#...","#....#..","........"})
GLYPH["Y"] = pack({"#....#..",".#..#...","..##....","..#.....","..#.....","..#.....","..#.....","........"})
GLYPH["Z"] = pack({"######..",".....#..","....#...","...#....","..#.....","#.......","######..","........"})
GLYPH["."] = pack({"........","........","........","........","........","........",".##.....","........"})
GLYPH[","] = pack({"........","........","........","........","........",".##.....",".#......","#......."})
GLYPH["!"] = pack({"..#.....","..#.....","..#.....","..#.....","..#.....","........","..#.....","........"})
GLYPH["?"] = pack({".####...","#....#..",".....#..","...##...","..#.....","........","..#.....","........"})
GLYPH["-"] = pack({"........","........","........","######..","........","........","........","........"})
GLYPH[":"] = pack({"........","..#.....","........","........","........","..#.....","........","........"})
GLYPH["/"] = pack({".....#..","....#...","...#....","..#.....","..#......",".#......","#.......","........"})
GLYPH["*"] = pack({"........","#.#.#...",".###....","#####...",".###....","#.#.#...","........","........"})
GLYPH["+"] = pack({"........","..#.....","..#.....","#####...","..#.....","..#.....","........","........"})
GLYPH["="] = pack({"........","........","######..","........","######..","........","........","........"})
GLYPH["("] = pack({"...#....","..#.....","..#.....","..#.....","..#.....","..#.....","...#....","........"})
GLYPH[")"] = pack({"..#.....","...#....","...#....","...#....","...#....","...#....","..#.....","........"})
GLYPH["%"] = pack({"##...#..","##..#...","...#....","..#.....","..#......",".#..##..","#...##..","........"})
GLYPH["'"] = pack({".#......",".#......","#.......","........","........","........","........","........"})
GLYPH["<"] = pack({"....#...","...#....","..#.....","#.......","..#.....","...#....","....#...","........"})
GLYPH[">"] = pack({"#.......",".#......","..#.....","...#....","..#.....","..#......","#.......","........"})

function G.textWidth(s, scale)
  scale = scale or 1
  return #s * 8 * scale
end

function G.print(s, x, y, col, scale)
  scale = scale or 1
  col = col or G.palette.white
  s = string.upper(tostring(s))
  love.graphics.setColor(col[1], col[2], col[3], col[4] or 1)
  local px = x
  for i = 1, #s do
    local g = GLYPH[s:sub(i, i)] or GLYPH[" "]
    for row = 0, 7 do
      local bits = g[row + 1]
      for c = 0, 7 do
        if math.floor(bits / 2 ^ (7 - c)) % 2 == 1 then
          love.graphics.rectangle("fill", px + c * scale, y + row * scale, scale, scale)
        end
      end
    end
    px = px + 8 * scale
  end
end

function G.printShadow(s, x, y, col, scale)
  G.print(s, x + scale, y + scale, G.palette.black, scale)
  G.print(s, x, y, col, scale)
end

function G.center(s, y, col, scale)
  scale = scale or 1
  local x0 = (G.viewLeft or 0)
  local x1 = (G.viewRight or 192)
  local maxW = math.max(8, x1 - x0)
  s = tostring(s)
  local w = G.textWidth(s, scale)
  if w > maxW then
    scale = maxW / math.max(1, #s * 8)
    w = G.textWidth(s, scale)
  end
  G.printShadow(s, math.floor(x0 + (maxW - w) / 2), y, col, scale)
end

G.viewLeft, G.viewRight = 0, 192

function G.setView(x0, x1)
  G.viewLeft = x0 or 0
  G.viewRight = x1 or 192
end

-- Visible horizontal span in playfield coords: x, width. Wider than the
-- 192 playfield on wide windows, narrower when the sides are cropped.
-- Use it for anything that should cover the whole screen (dim overlays,
-- bands, flashes) instead of a hard-coded 0..192.
function G.viewSpan()
  local x0 = math.floor((G.viewLeft or 0) - 4)
  local x1 = math.ceil((G.viewRight or 192) + 4)
  return x0, math.max(1, x1 - x0)
end

-- One top-down ground map, straight vertical scroll (Raiden).
function G.drawParallax(assets, scroll)
  local h = 256
  local img = assets.bg or assets.bgApt
  if not img then
    return
  end
  local y = (scroll % h)
  love.graphics.setColor(1, 1, 1, 1)
  love.graphics.draw(img, 0, y)
  love.graphics.draw(img, 0, y - h)
end

-- Title logo "CAUSEWAYBAY RAIDEN"
function G.drawLogo(x, y, t)
  local rust = G.palette.rust
  local yel = G.palette.yellow
  local cyn = G.palette.cyan
  local flash = (math.floor(t * 8) % 2 == 0) and yel or rust
  G.printShadow("CAUSEWAYBAY", x, y, flash, 1)
  G.printShadow("RAIDEN", x + 8, y + 16, cyn, 2)
end

return G
