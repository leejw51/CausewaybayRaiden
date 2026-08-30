-- Aspect-correct 192x256 playfield (uniform scale).
-- Extra window space is filled with scrolling stage art, not empty bars.

local D = {}

D.GW, D.GH = 192, 256
D.layout = "vertical" -- "vertical" | "horizontal"
D.fullscreen = true
D.windowScale = 3
D.canvas = nil
D.ox, D.oy, D.scale = 0, 0, 3
D.scaleX, D.scaleY = 3, 3
D.shakeX, D.shakeY = 0, 0
D.uiButtons = {}

local function flags()
  return {
    fullscreen = D.fullscreen,
    fullscreentype = "desktop",
    vsync = 1,
    -- Resizable even in fullscreen. SDL only puts a resizable window into a
    -- native macOS fullscreen Space; a fixed-size one falls back to legacy
    -- fullscreen at CGShieldingWindowLevel, which paints over the
    -- Shift+Command+5 capture UI and hides it.
    resizable = true,
    minwidth = D.GW,
    minheight = D.GH,
    msaa = 0,
    highdpi = false,
    usedpiscale = false,
  }
end

function D.init()
  love.graphics.setDefaultFilter("nearest", "nearest")
  love.graphics.setLineStyle("rough")
  D.canvas = love.graphics.newCanvas(D.GW, D.GH)
  D.canvas:setFilter("nearest", "nearest")
  D.applyWindow()
end

function D.toggleLayout()
  D.layout = (D.layout == "vertical") and "horizontal" or "vertical"
  D.applyWindow()
end

function D.toggleFullscreen()
  D.fullscreen = not D.fullscreen
  D.applyWindow()
end

function D.applyWindow()
  local dw, dh = love.window.getDesktopDimensions()
  if D.fullscreen then
    -- setFullscreen keeps the window and its flags, so macOS keeps the Space.
    -- setMode here could recreate the window and drop it back to legacy mode.
    love.window.setFullscreen(true, "desktop")
    return
  end
  local maxW = math.max(D.GW, dw - 24)
  local maxH = math.max(D.GH, dh - 72)
  if D.layout == "vertical" then
    -- True 3:4 window so the playfield is never cropped.
    local s = math.max(2, math.min(math.floor(maxW / D.GW), math.floor(maxH / D.GH)))
    D.windowScale = s
    love.window.setMode(D.GW * s, D.GH * s, flags())
  else
    local s = math.max(2, math.min(math.floor(maxW / (D.GW + 96)), math.floor(maxH / D.GH)))
    D.windowScale = s
    local playH = D.GH * s
    local w = math.min(maxW, math.max(D.GW * s + 220, math.floor(playH * 16 / 9)))
    love.window.setMode(w, playH, flags())
  end
end

function D.syncLayout()
  local sw, sh = love.graphics.getDimensions()
  -- Fill the window HEIGHT so vertical mode has no top/bottom bands.
  -- 3:4 is kept (uniform scale). Extra width is background; if the
  -- window is taller than 3:4, sides of the playfield are cropped.
  local s = math.max(1, sh / D.GH)
  local x = math.floor((sw - D.GW * s) / 2)
  local y = 0
  D.scale = s
  D.scaleX, D.scaleY = s, s
  D.ox, D.oy = x, y
  D.viewLeft = math.max(0, math.ceil(-x / s))
  D.viewRight = math.min(D.GW, math.floor((sw - x) / s))
  return sw, sh, s, x, y
end

function D.begin()
  D.syncLayout()
  local G = package.loaded["src.gfx"]
  if G and G.setView then
    G.setView((D.viewLeft or 0) + 3, (D.viewRight or 192) - 3)
  end
  love.graphics.setCanvas(D.canvas)
  love.graphics.clear(0, 0, 0, 0)
end

function D.placeButtons(sw, sh)
  local fs = math.max(4, math.min(6, math.floor(sw / 180)))
  local bw = 4 * 8 * fs + 16
  local bh = 8 * fs + 8
  local by = 8
  D.uiButtons = {
    { id = "layout", x = 8, y = by, w = bw, h = bh, fs = fs },
    { id = "full", x = sw - bw - 8, y = by, w = bw, h = bh, fs = fs },
  }
end

function D.hitButton(mx, my)
  for _, b in ipairs(D.uiButtons) do
    if mx >= b.x and mx < b.x + b.w and my >= b.y and my < b.y + b.h then
      return b.id
    end
  end
  return nil
end

function D.tileBackdrop(img, ox, oy, tw, th, sw, sh, r, g, b, a)
  if not img then
    return
  end
  love.graphics.setColor(r, g, b, a)
  local startX = ox
  while startX > 0 do
    startX = startX - tw
  end
  local startY = oy
  while startY > 0 do
    startY = startY - th
  end
  for ty = startY, sh, th do
    for tx = startX, sw, tw do
      love.graphics.draw(img, tx, ty, 0, tw / img:getWidth(), th / img:getHeight())
    end
  end
end

function D.drawWrap(img, mul, scroll, sw, sh, s, x, y, r, g, b, a)
  if not img then
    return
  end
  img:setWrap("repeat", "repeat")
  local iw, ih = img:getWidth(), img:getHeight()
  local v0 = (-scroll * mul) % ih
  if v0 < 0 then
    v0 = v0 + ih
  end
  v0 = math.floor(v0)
  local u0 = -x / s
  local q = love.graphics.newQuad(u0, v0, sw / s, sh / s, iw, ih)
  love.graphics.setColor(r or 1, g or 1, b or 1, a or 1)
  love.graphics.draw(img, q, 0, y, 0, s, s)
end

function D.finish(shake, bg)
  shake = shake or 0
  bg = bg or {}
  love.graphics.setCanvas()
  local sw, sh, s, x, y = D.syncLayout()

  local jx, jy = 0, 0
  if shake > 0 then
    jx = math.floor((love.math.random() * 2 - 1) * shake)
    jy = math.floor((love.math.random() * 2 - 1) * shake)
  end
  D.shakeX, D.shakeY = jx, jy

  -- Raiden-style vertical parallax: far < ground < clouds.
  love.graphics.clear(0.04, 0.06, 0.12, 1)
  local scroll = bg.scroll or 0
  local ground = bg.city or bg.mid
  D.drawWrap(bg.far or ground, 0.32, scroll, sw, sh, s, x, y, 0.62, 0.66, 0.82, 1)
  D.drawWrap(ground, 1.0, scroll, sw, sh, s, x, y, 1, 1, 1, 1)
  love.graphics.setBlendMode("alpha")
  D.drawWrap(bg.haze, 0.42, scroll, sw, sh, s, x, y, 1, 1, 1, bg.hazeA or 0.34)
  D.drawWrap(bg.clouds or bg.near, 1.68, scroll, sw, sh, s, x, y, 1, 1, 1, bg.cloudA or 0.48)

  love.graphics.setColor(1, 1, 1, 1)
  love.graphics.setBlendMode("alpha", "premultiplied")
  love.graphics.draw(D.canvas, x + jx, y + jy, 0, s, s)
  love.graphics.setBlendMode("alpha")

  love.graphics.setColor(0, 0, 0, 0.07)
  for lineY = 0, sh - 1, 2 do
    love.graphics.rectangle("fill", 0, lineY, sw, 1)
  end

  D.placeButtons(sw, sh)
  return x, y, s, sw, sh
end

function D.toScreen(px, py)
  return D.ox + px * D.scaleX, D.oy + py * D.scaleY
end

return D
