function love.conf(t)
  t.identity = "bugraiden"
  t.version = "11.5"
  t.console = false
  t.accelerometerjoystick = false
  t.gammacorrect = false

  t.window.title = "CAUSEWAYBAY RAIDEN"
  t.window.icon = nil
  t.window.width = 576
  t.window.height = 768
  t.window.minwidth = 192
  t.window.minheight = 256
  t.window.resizable = true
  t.window.vsync = 1
  t.window.msaa = 0
  t.window.highdpi = false
  t.window.usedpiscale = false
  t.window.display = 1
  -- Fullscreen from creation, matching Display.fullscreen. Together with
  -- resizable above this makes SDL open a native macOS fullscreen Space, so
  -- Shift+Command+5 can still draw its capture UI over the game.
  t.window.fullscreen = true
  t.window.fullscreentype = "desktop"

  t.modules.physics = false
  t.modules.video = false
  t.modules.thread = false
end
