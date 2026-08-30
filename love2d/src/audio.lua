-- Tiny 8-bit synth: square / triangle / noise, MSX-ish chiptunes.

local Audio = {}

local SR = 22050
local sources = {}
local music = { title = nil, stage = nil, boss = nil, over = nil }
local current = nil

local function newData(n)
  return love.sound.newSoundData(n, SR, 16, 1)
end

local function clamp(v)
  if v > 1 then return 1 end
  if v < -1 then return -1 end
  return v
end

local function envAR(i, n, atk, rel)
  local a = math.max(1, math.floor(SR * atk))
  local r = math.max(1, math.floor(SR * rel))
  if i < a then
    return i / a
  end
  if i > n - r then
    return math.max(0, (n - i) / r)
  end
  return 1
end

local function square(phase, duty)
  return (phase % 1 < duty) and 1 or -1
end

local function tri(phase)
  local p = phase % 1
  if p < 0.5 then
    return p * 4 - 1
  end
  return 3 - p * 4
end

local function noise()
  return love.math.random() * 2 - 1
end

local function pulseTone(freq, dur, vol, duty, decay)
  duty = duty or 0.25
  decay = decay or 1
  local n = math.max(1, math.floor(SR * dur))
  local d = newData(n)
  for i = 0, n - 1 do
    local t = i / SR
    local e = envAR(i, n, 0.004, 0.02) * (1 - decay * t / dur)
    if e < 0 then e = 0 end
    d:setSample(i, clamp(square(t * freq, duty) * vol * e))
  end
  local src = love.audio.newSource(d, "static")
  return src
end

local function noiseBurst(dur, vol, decay)
  local n = math.max(1, math.floor(SR * dur))
  local d = newData(n)
  local last = 0
  for i = 0, n - 1 do
    local t = i / SR
    -- Cheap NES-ish: hold noise for a few samples
    if i % 3 == 0 then
      last = noise()
    end
    local e = (1 - t / dur) ^ decay
    d:setSample(i, clamp(last * vol * e))
  end
  return love.audio.newSource(d, "static")
end

local function sweep(f0, f1, dur, vol, duty)
  local n = math.max(1, math.floor(SR * dur))
  local d = newData(n)
  local phase = 0
  for i = 0, n - 1 do
    local u = i / n
    local f = f0 + (f1 - f0) * u
    phase = phase + f / SR
    local e = envAR(i, n, 0.003, 0.03) * (1 - u * 0.4)
    d:setSample(i, clamp(square(phase, duty or 0.25) * vol * e))
  end
  return love.audio.newSource(d, "static")
end

local function arp(freqs, step, vol)
  local n = math.max(1, math.floor(SR * step * #freqs))
  local d = newData(n)
  local stepN = math.floor(SR * step)
  for i = 0, n - 1 do
    local idx = math.min(#freqs, math.floor(i / stepN) + 1)
    local f = freqs[idx]
    local t = i / SR
    local e = envAR(i % stepN, stepN, 0.002, 0.01)
    d:setSample(i, clamp(square(t * f, 0.25) * vol * e))
  end
  return love.audio.newSource(d, "static")
end

-- Note table
local N = {}
do
  local names = { "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B" }
  for oct = 1, 6 do
    for i, name in ipairs(names) do
      local midi = 12 * (oct + 1) + (i - 1)
      N[name .. oct] = 440 * 2 ^ ((midi - 69) / 12)
    end
  end
end

local function parseToken(tok)
  if not tok or tok == "." or tok == "-" then
    return 0
  end
  return N[tok] or 0
end

local function renderSong(bpm, pattern, vol)
  -- pattern = { {wave, duty, tokens = {"A3","A3",...} }, ... }
  local spb = 60 / bpm / 4 -- 16th
  local len = #pattern[1].notes
  local n = math.max(1, math.floor(SR * spb * len))
  local d = newData(n)
  local acc = {}
  for i = 0, n - 1 do
    acc[i] = 0
  end

  for _, ch in ipairs(pattern) do
    local stepN = math.floor(SR * spb)
    local phase = 0
    for i = 0, n - 1 do
      local ni = math.floor(i / stepN)
      local nts = ch.notes
      local f = parseToken(nts[(ni % #nts) + 1])
      local e = envAR(i % stepN, stepN, 0.003, 0.012) * (ch.vol or 1)
      local s = 0
      if f > 0 then
        phase = phase + f / SR
        if ch.wave == "tri" then
          s = tri(phase)
        elseif ch.wave == "noise" then
          if i % 4 == 0 then
            s = noise() * 0.7
          end
        else
          s = square(phase, ch.duty or 0.25)
        end
      else
        phase = 0
      end
      acc[i] = acc[i] + s * e * vol
    end
  end

  for i = 0, n - 1 do
    d:setSample(i, clamp(acc[i]))
  end
  local src = love.audio.newSource(d, "static")
  src:setLooping(true)
  return src
end

local function notes(str)
  local t = {}
  for tok in string.gmatch(str, "%S+") do
    t[#t + 1] = tok
  end
  return t
end

function Audio.init()
  sources.shot = pulseTone(1244, 0.045, 0.18, 0.125, 0.4)
  sources.shot2 = pulseTone(1568, 0.04, 0.12, 0.25, 0.6)
  sources.hit = pulseTone(220, 0.04, 0.22, 0.5, 1.2)
  sources.coin = arp({ 988, 1318, 1568 }, 0.05, 0.22)
  sources.start = arp({ 523, 659, 784, 1046, 784, 1046, 1318 }, 0.07, 0.2)
  sources.power = arp({ 392, 523, 659, 784, 1046 }, 0.055, 0.22)
  sources.oneup = arp({ 523, 659, 784, 1046, 1318, 1568, 2093 }, 0.07, 0.2)
  sources.bomb = sweep(120, 40, 0.55, 0.28, 0.5)
  sources.explode = noiseBurst(0.28, 0.32, 1.4)
  sources.explodeBig = noiseBurst(0.55, 0.38, 1.1)
  sources.death = sweep(440, 80, 0.7, 0.26, 0.25)
  sources.warn = sweep(440, 880, 0.35, 0.2, 0.25)
  sources.warn2 = sweep(880, 440, 0.35, 0.2, 0.25)
  sources.blip = pulseTone(880, 0.05, 0.16, 0.25, 0.2)
  sources.graze = pulseTone(1760, 0.03, 0.08, 0.125, 0.8)
  sources.select = pulseTone(660, 0.06, 0.16, 0.25, 0.3)

  -- Title: bright Konami-ish C major
  music.title = renderSong(150, {
    {
      wave = "square", duty = 0.25, vol = 0.55,
      notes = notes([[
        C5 . E5 . G5 . C6 G5 E5 C5 A4 . C5 . E5 . A5 E5
        F5 . A5 . C6 . A5 F5 . G5 . B5 . D6 C6 G5 .
        C5 . E5 . G5 . C6 G5 E5 C5 A4 . C5 . E5 . A5 E5
        F5 A5 C6 . G5 B5 D6 . C6 . . . G5 . . . C5 .
      ]]),
    },
    {
      wave = "square", duty = 0.5, vol = 0.28,
      notes = notes([[
        E4 . . . G4 . . . E4 . . . C4 . . . F4 . . . A4 . . . D4 . . . G4 . . .
        E4 . . . G4 . . . E4 . . . C4 . . . F4 . . . G4 . . . E4 . . . . . . .
      ]]),
    },
    {
      wave = "tri", vol = 0.45,
      notes = notes([[
        C3 C3 C3 C3 G2 G2 G2 G2 A2 A2 A2 A2 E2 E2 E2 E2
        F2 F2 F2 F2 C3 C3 C3 C3 G2 G2 G2 G2 C3 C3 G2 G2
        C3 C3 C3 C3 G2 G2 G2 G2 A2 A2 A2 A2 E2 E2 E2 E2
        F2 F2 F2 F2 G2 G2 G2 G2 C3 . . . G2 . . . C3 .
      ]]),
    },
    {
      wave = "noise", vol = 0.18,
      notes = notes([[
        C3 . C3 . C3 . C3 C3 C3 . C3 . C3 . C3 C3
        C3 . C3 . C3 . C3 C3 C3 . C3 . C3 C3 C3 .
        C3 . C3 . C3 . C3 C3 C3 . C3 . C3 . C3 C3
        C3 . C3 . C3 . C3 C3 C3 . . . C3 . . .
      ]]),
    },
  }, 0.22)

  -- Stage: A-minor driving raid theme
  music.stage = renderSong(138, {
    {
      wave = "square", duty = 0.125, vol = 0.5,
      notes = notes([[
        A4 . C5 E5 . D5 C5 B4 A4 . E4 . A4 . C5 B4
        G4 . B4 D5 . C5 B4 A4 G4 . D4 . G4 . B4 A4
        F4 . A4 C5 . B4 A4 G4 F4 . C4 . F4 . A4 G4
        E4 . G4 B4 . A4 G4 F4 E4 . B3 . E4 . G4 E4
      ]]),
    },
    {
      wave = "square", duty = 0.5, vol = 0.22,
      notes = notes([[
        E4 . . A4 . . E4 . D4 . . G4 . . D4 .
        C4 . . F4 . . C4 . B3 . . E4 . . B3 .
      ]]),
    },
    {
      wave = "tri", vol = 0.5,
      notes = notes([[
        A2 A2 E3 E3 A2 A2 E3 E3 G2 G2 D3 D3 G2 G2 D3 D3
        F2 F2 C3 C3 F2 F2 C3 C3 E2 E2 B2 B2 E2 E2 G2 G2
      ]]),
    },
    {
      wave = "noise", vol = 0.16,
      notes = notes([[
        C3 . C3 C3 C3 . C3 . C3 . C3 C3 C3 . C3 C3
        C3 . C3 C3 C3 . C3 . C3 . C3 C3 C3 C3 C3 .
      ]]),
    },
  }, 0.2)

  -- Boss: faster, harsher
  music.boss = renderSong(168, {
    {
      wave = "square", duty = 0.25, vol = 0.55,
      notes = notes([[
        E4 F4 F#4 G4 . . G4 . D#4 E4 F4 F#4 . . F#4 .
        G4 G#4 A4 A#4 . A4 G4 . E4 . G4 . B4 . E5 .
        E4 F4 F#4 G4 . . G4 . D#4 E4 F4 F#4 . . F#4 .
        C5 . B4 . A#4 . A4 . G4 . E4 . D4 . E4 .
      ]]),
    },
    {
      wave = "tri", vol = 0.48,
      notes = notes([[
        E2 E2 E3 E3 E2 E2 B2 B2 D#2 D#2 D#3 D#3 D#2 D#2 A#2 A#2
        G2 G2 G3 G3 G2 G2 D3 D3 E2 E2 B2 B2 E2 E2 E3 E3
      ]]),
    },
    {
      wave = "noise", vol = 0.22,
      notes = notes([[
        C3 C3 C3 . C3 C3 C3 . C3 C3 C3 . C3 C3 C3 C3
        C3 C3 C3 . C3 C3 C3 . C3 . C3 . C3 C3 C3 .
      ]]),
    },
  }, 0.22)

  music.over = renderSong(90, {
    {
      wave = "square", duty = 0.5, vol = 0.45,
      notes = notes([[
        E5 . D5 . C5 . B4 . A4 . G4 . E4 . . . C4 . . .
        D4 . E4 . G4 . E4 . D4 . . . C4 . . . . . . .
      ]]),
    },
    {
      wave = "tri", vol = 0.4,
      notes = notes([[
        A2 . . . E2 . . . F2 . . . C2 . . . G2 . . . D2 . . . A2 . . . . . . .
      ]]),
    },
  }, 0.18)
  music.over:setLooping(false)
end

local function playSrc(src, vol)
  if not src then return end
  local s = src:clone()
  s:setVolume(vol or 1)
  s:play()
end

local reuse = {
  shot = true, shot2 = true, hit = true, graze = true, blip = true, select = true,
}

function Audio.play(name)
  local src = sources[name]
  if not src then return end
  if reuse[name] then
    src:stop()
    src:setVolume(0.9)
    src:play()
  else
    playSrc(src, 0.9)
  end
end

function Audio.warn()
  playSrc(sources.warn, 1)
  playSrc(sources.warn2, 0.8)
end

function Audio.music(name)
  if current == name then
    return
  end
  if current and music[current] then
    music[current]:stop()
  end
  current = name
  if name and music[name] then
    music[name]:setVolume(0.55)
    music[name]:play()
  end
end

function Audio.stopMusic()
  for _, m in pairs(music) do
    m:stop()
  end
  current = nil
end

return Audio
