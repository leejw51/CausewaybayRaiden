-- Three Raiden-style stages. Times in seconds from stage start.

local Stage = {}

Stage.NAMES = {
  "CAUSEWAYBAY",
  "MTR LINE",
  "HKU CAMPUS",
}

Stage.SUB = {
  "HENNESSY RD",
  "DEADLOCK EXPRESS",
  "BURGER RUN",
}

Stage.WARN = {
  { "WARNING", "STACK", "OVERFLOW" },
  { "WARNING", "THREAD", "DEADLOCK" },
  { "WARNING", "SEGMENTATION", "FAULT" },
}

function Stage.build(stage, loop)
  local L = loop or 1
  stage = stage or 1
  if stage == 2 then
    return Stage.mtr(L)
  elseif stage == 3 then
    return Stage.hku(L)
  end
  return Stage.apt(L)
end

function Stage.apt(L)
  return {
    { 1.8, function(w)
      w.phaseName = "HENNESSY RD"
      w:spawnV("beetle", 3, 96, -14, 22)
      w:dropItem(48, -8, "C")
    end },
    { 4.0, function(w)
      w:spawnLine("beetle", 3, 28, 28, -10, "sine")
      w:spawn("offby1", 96, -16, { path = "zigzag", vx = 36, drop = "P" })
    end },
    { 6.2, function(w)
      w:spawnLine("beetle", 3, 164, 164, -10, "sine")
    end },
    { 8.0, function(w)
      w:spawn("moth", 48, -18, { path = "swoop" })
      w:spawn("moth", 144, -18, { path = "swoop" })
      w:dropItem(144, -8, "G")
    end },
    { 10.5, function(w)
      w:spawn("nullptr", 24, -10, { path = "dive" })
      w:spawn("nullptr", 168, -10, { path = "dive" })
      w:spawn("clippy", 96, -28, { path = "hover", drop = "A" })
    end },
    { 13.5, function(w)
      w.phaseName = "HYSAN PLACE"
      w:spawn("leak", 64, -18, { path = "hover" })
      w:spawnV("beetle", 3, 96, -10, 20)
    end },
    { 16.5, function(w)
      w:spawn("overflow", 96, -28, { path = "down", drop = "T" })
      w:spawn("moth", 40, -16, { path = "swoop" })
      w:spawn("moth", 152, -16, { path = "swoop" })
    end },
    { 20.0, function(w)
      for i = 0, 3 do
        w:spawn("beetle", 28 + i * 36, -10, { path = "down", speed = 48 })
      end
      w:spawn("lifetime", 96, -22, { path = "teleport", drop = "U" })
      w:dropItem(96, -8, "X")
    end },
    { 23.5, function(w)
      w:spawn("overflow", 96, -36, { path = "hover", midboss = true, drop = "P" })
    end },
    { 28.0, function(w)
      w:spawnV("beetle", 3, 96, -12, 18)
      w:spawn("panic", 40, -14, { path = "dive" })
      w:spawn("panic", 152, -14, { path = "dive" })
    end },
    { 32.0, function(w)
      w.warningT = 3.2
      w.bossWarn = Stage.WARN[1]
      w.audio.play("warn")
      w.audio.play("warn2")
    end },
    { 35.6, function(w)
      w.audio.music("boss")
      w:spawn("bossOverflow", 96, -70, { path = "boss" })
      w.shake = 5
    end },
  }
end

function Stage.mtr(L)
  return {
    { 1.6, function(w)
      w.phaseName = "MTR TUNNEL"
      w:spawnLine("beetle", 3, 32, 160, -8, "down")
      w:dropItem(64, -8, "G")
    end },
    { 4.2, function(w)
      w:spawn("deadlock", -16, 8, { path = "side", vx = 42 })
      w:spawn("deadlock", 208, 18, { path = "side", vx = -42, drop = "P" })
    end },
    { 7.0, function(w)
      w:spawn("spider", 96, -18, { path = "hover", drop = "A" })
    end },
    { 10.0, function(w)
      w:spawn("nullptr", 48, -10, { path = "dive" })
      w:spawn("nullptr", 144, -10, { path = "dive" })
    end },
    { 12.8, function(w)
      w:spawn("infloop", 96, -22, { path = "orbit" })
      w:spawnLine("beetle", 3, 30, 40, -10, "sine")
    end },
    { 16.0, function(w)
      w:spawn("deadlock", 96, -24, { path = "hover", drop = "B" })
      w:spawn("moth", 32, -16, { path = "swoop" })
      w:spawn("moth", 160, -16, { path = "swoop" })
      w:dropItem(128, -8, "X")
    end },
    { 19.5, function(w)
      w.phaseName = "LOCKED THREADS"
      w:spawn("deadlock", 96, -18, { path = "hover" })
      w:spawn("heisen", 96, -14, { path = "teleport" })
    end },
    { 23.0, function(w)
      w:spawn("worm", 96, -22, { path = "sine" })
      w:spawn("clippy", 96, -32, { path = "hover", drop = "T" })
    end },
    { 27.0, function(w)
      w:spawn("deadlock", 96, -32, { path = "hover", midboss = true, drop = "A" })
    end },
    { 32.0, function(w)
      for i = 0, 3 do
        w:spawn("beetle", 32 + i * 40, -8 - (i % 2) * 10, { path = "zigzag", vx = (i % 2 == 0) and 32 or -40 })
      end
    end },
    { 35.5, function(w)
      w.warningT = 3.2
      w.bossWarn = Stage.WARN[2]
      w.audio.play("warn")
    end },
    { 39.0, function(w)
      w.audio.music("boss")
      w:spawn("bossDeadlock", 96, -72, { path = "boss" })
      w.shake = 6
    end },
  }
end

function Stage.hku(L)
  return {
    { 1.5, function(w)
      w.phaseName = "CAMPUS RUN"
      w:spawnV("beetle", 3, 96, -12, 20)
      w:spawn("panic", 96, -12, { path = "dive" })
      w:dropItem(96, -8, "X")
    end },
    { 4.4, function(w)
      w:spawn("overflow", 48, -20, { path = "zigzag", vx = 28 })
      w:spawn("overflow", 144, -20, { path = "zigzag", vx = -28, drop = "P" })
    end },
    { 7.2, function(w)
      w:spawn("deadlock", -14, 6, { path = "side", vx = 50 })
      w:spawn("leak", 96, -18, { path = "hover", drop = "A" })
    end },
    { 10.5, function(w)
      w:spawn("moth", 48, -16, { path = "swoop" })
      w:spawn("moth", 144, -16, { path = "swoop" })
      w:dropItem(48, -8, "C")
    end },
    { 14.0, function(w)
      w:spawn("nullptr", 48, -10, { path = "dive" })
      w:spawn("nullptr", 144, -10, { path = "dive" })
    end },
    { 17.5, function(w)
      w.phaseName = "KERNEL EDGE"
      w:spawn("infloop", 96, -20, { path = "orbit", drop = "T" })
      w:spawn("clippy", 60, -24, { path = "hover" })
      w:spawn("lifetime", 132, -24, { path = "teleport" })
    end },
    { 21.5, function(w)
      w:spawn("overflow", 96, -34, { path = "hover", midboss = true, drop = "P" })
    end },
    { 26.5, function(w)
      w:spawnV("beetle", 4, 96, -10, 18)
      w:spawn("panic", 96, -12, { path = "dive" })
    end },
    { 30.5, function(w)
      w:spawn("spider", 96, -20, { path = "hover", drop = "B" })
      w:spawnLine("beetle", 4, 32, 160, -8, "dive")
    end },
    { 34.5, function(w)
      w.warningT = 3.6
      w.bossWarn = Stage.WARN[3]
      w.audio.play("warn")
      w.audio.play("warn2")
    end },
    { 38.4, function(w)
      w.audio.music("boss")
      w:spawn("boss", 96, -78, { path = "boss" })
      w.shake = 7
    end },
  }
end

return Stage
