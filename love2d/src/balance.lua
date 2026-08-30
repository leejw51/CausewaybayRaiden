-- Difficulty ranks and combat knobs for CAUSEWAYBAY RAIDEN.

local Balance = {}

Balance.LEVELS = { "easy", "normal", "hard" }

Balance.INFO = {
  easy = { title = "EASY", sub = "TRAINING BUILD" },
  normal = { title = "NORMAL", sub = "RELEASE BUILD" },
  hard = { title = "HARD", sub = "DEBUG HELL" },
}

Balance.DEFAULT = "normal"

-- Higher fireRateMul / bulletSpdMul / hpMul = harder.
-- Agents: high agentRateMul = slower shots; high agentEatMul = better blocking.
local DIFF = {
  easy = {
    hpMul = 0.80,
    spdMul = 0.82,
    fireRateMul = 0.78,
    bulletSpdMul = 0.82,
    bossHpMul = 0.62,
    lives = 5,
    bombs = 5,
    power = 2,
    agentDmgMul = 0.90,
    agentRateMul = 0.22,
    agentEatMul = 1.05,
    agentAutoFire = true,
    tokenRainEmpty = 4.2,
    tokenRainBusy = 7.2,
  },
  normal = {
    hpMul = 1.20,
    spdMul = 1.08,
    fireRateMul = 1.28,
    bulletSpdMul = 1.18,
    bossHpMul = 1.00,
    lives = 3,
    bombs = 3,
    power = 1,
    agentDmgMul = 0.38,
    agentRateMul = 1.00,
    agentEatMul = 1.25,
    agentAutoFire = false,
    tokenRainEmpty = 6.4,
    tokenRainBusy = 10.5,
  },
  hard = {
    hpMul = 1.65,
    spdMul = 1.26,
    fireRateMul = 1.65,
    bulletSpdMul = 1.42,
    bossHpMul = 1.55,
    lives = 2,
    bombs = 2,
    power = 1,
    agentDmgMul = 0.24,
    agentRateMul = 1.30,
    agentEatMul = 1.40,
    agentAutoFire = false,
    tokenRainEmpty = 8.5,
    tokenRainBusy = 14.0,
  },
}

function Balance.normalize(id)
  if type(id) == "number" then
    id = Balance.LEVELS[id]
  end
  if type(id) ~= "string" then
    return Balance.DEFAULT
  end
  id = string.lower(id)
  if DIFF[id] then
    return id
  end
  return Balance.DEFAULT
end

function Balance.get(id)
  local key = Balance.normalize(id)
  local src = DIFF[key]
  local out = { id = key }
  for k, v in pairs(src) do
    out[k] = v
  end
  local info = Balance.INFO[key]
  out.title = info.title
  out.sub = info.sub
  return out
end

function Balance.label(id)
  return Balance.INFO[Balance.normalize(id)].title
end

function Balance.index(id)
  id = Balance.normalize(id)
  for i, k in ipairs(Balance.LEVELS) do
    if k == id then
      return i
    end
  end
  return 2
end

function Balance.harder(a, b)
  return Balance.index(a) > Balance.index(b)
end

return Balance
