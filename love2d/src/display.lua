-- Virtual-resolution display, done the usual LÖVE way:
--
--   * one fixed virtual HEIGHT (GH = 256) and a uniform scale s = winH / GH,
--     so pixels are never stretched;
--   * the virtual WIDTH follows the window's aspect ratio, so the canvas
--     always covers the whole window - no bars, no letterbox;
--   * the 192x256 playfield (GW x GH, the gameplay safe area) sits centred
--     in that canvas. Extra width simply shows more of the same world
--     (ground, clouds, props, enemies flying in). If the window is taller
--     than 3:4 the playfield's sides are cropped instead;
--   * layout is recomputed from love.resize, not every frame, and the
--     canvas is rebuilt only when the virtual size actually changes.
--
-- World code keeps drawing in 0..192 playfield coordinates: D.begin()
-- translates by D.px so the playfield lands in the middle of the canvas.
-- D.viewLeft / D.viewRight (playfield coords) say what is actually visible;
-- they go negative / past 192 in wide mode and shrink when cropping.

local D = {}

D.GW, D.GH = 192, 256 -- playfield (safe area); never changes
D.VW = 192            -- virtual width currently rendered (>= GW)
D.px = 0              -- playfield x offset inside the canvas
D.fullscreen = true
D.windowScale = 3
D.canvas = nil
D.ox, D.oy, D.scale = 0, 0, 3 -- playfield origin on screen, uniform scale
D.scaleX, D.scaleY = 3, 3
D.cx, D.cy = 0, 0             -- canvas origin on screen
D.sw, D.sh = 0, 0             -- window size the layout was computed for
D.viewLeft, D.viewRight = 0, 192
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
  D.applyWindow()
  D.resize(love.graphics.getDimensions())
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
    if love.window.isOpen() then
      love.window.setFullscreen(true, "desktop")
    else
      love.window.setMode(D.GW * D.windowScale, D.GH * D.windowScale, flags())
    end
  else
    -- Windowed preset: a true 3:4 window at the largest integer scale that
    -- fits the desktop. The window stays resizable; any other shape works.
    local maxW = math.max(D.GW, dw - 24)
    local maxH = math.max(D.GH, dh - 72)
    local s = math.max(2, math.min(math.floor(maxW / D.GW), math.floor(maxH / D.GH)))
    D.windowScale = s
    love.window.setMode(D.GW * s, D.GH * s, flags())
  end
  -- love.resize is not guaranteed after setMode/setFullscreen; sync now.
  D.resize(love.graphics.getDimensions())
end

-- Recompute the virtual resolution for a window size. Called from
-- love.resize and after our own mode changes.
function D.resize(sw, sh)
  sw, sh = math.max(1, math.floor(sw or 1)), math.max(1, math.floor(sh or 1))
  local s = math.max(1, sh / D.GH)
  -- Canvas is at least the playfield; wider when the window is wider than 3:4.
  local cw = math.max(D.GW, math.ceil(sw / s))
  D.VW = cw
  D.px = math.floor((cw - D.GW) / 2)
  D.scale = s
  D.scaleX, D.scaleY = s, s
  D.cx = math.floor((sw - cw * s) / 2) -- 0 in wide mode, negative when cropping
  D.cy = 0
  D.ox, D.oy = D.cx + D.px * s, 0
  D.viewLeft = math.max(-D.px, math.ceil(-D.ox / s))
  D.viewRight = math.min(cw - D.px, math.floor((sw - D.ox) / s))
  D.sw, D.sh = sw, sh
  if not D.canvas or D.canvas:getWidth() ~= cw or D.canvas:getHeight() ~= D.GH then
    D.canvas = love.graphics.newCanvas(cw, D.GH)
    D.canvas:setFilter("nearest", "nearest")
  end
  D.placeButtons(sw, sh)
end

-- Cheap guard for frames drawn before love.resize fires (e.g. during the
-- macOS fullscreen animation). Normally a no-op.
function D.syncLayout()
  local sw, sh = love.graphics.getDimensions()
  if sw ~= D.sw or sh ~= D.sh or not D.canvas then
    D.resize(sw, sh)
  end
  return sw, sh, D.scale, D.ox, D.oy
end

function D.begin()
  D.syncLayout()
  local G = package.loaded["src.gfx"]
  if G and G.setView then
    G.setView(D.viewLeft + 3, D.viewRight - 3)
  end
  love.graphics.setCanvas(D.canvas)
  love.graphics.clear(0, 0, 0, 0)
  love.graphics.push()
  love.graphics.translate(D.px, 0)
end

function D.placeButtons(sw, sh)
  local fs = math.max(2, math.floor((D.scale or 3) * 0.6 + 0.5))
  local bw = 4 * 8 * fs + 16
  local bh = 8 * fs + 8
  local by = 8
  D.uiButtons = {
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
  love.graphics.pop()
  love.graphics.setCanvas()
  local sw, sh, s, x, y = D.syncLayout()

  local jx, jy = 0, 0
  if shake > 0 then
    jx = math.floor((love.math.random() * 2 - 1) * shake)
    jy = math.floor((love.math.random() * 2 - 1) * shake)
  end
  D.shakeX, D.shakeY = jx, jy

  -- Raiden-style vertical parallax: far < ground < clouds. Drawn in window
  -- space across the whole window, anchored to the playfield origin.
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
  love.graphics.draw(D.canvas, D.cx + jx, D.cy + jy, 0, s, s)
  love.graphics.setBlendMode("alpha")

  love.graphics.setColor(0, 0, 0, 0.07)
  for lineY = 0, sh - 1, 2 do
    love.graphics.rectangle("fill", 0, lineY, sw, 1)
  end

  return x, y, s, sw, sh
end

-- Playfield coords -> window coords.
function D.toScreen(px, py)
  return D.ox + px * D.scaleX, D.oy + py * D.scaleY
end

-- Window coords -> playfield coords.
function D.toGame(sx, sy)
  return (sx - D.ox) / D.scaleX, (sy - D.oy) / D.scaleY
end

return D
