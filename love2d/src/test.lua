-- Lightweight in-engine tests. Invoked by:  love . -- --test

local M = {}

function M.run(assets, Audio, Display)
  local World = require "src.world"
  local Stage = require "src.stage"
  local fails = 0
  local n = 0

  local function check(name, cond)
    n = n + 1
    if cond then
      print("ok " .. n .. " - " .. name)
    else
      fails = fails + 1
      print("not ok " .. n .. " - " .. name)
    end
  end

  check("playfield is 192x256", Display.GW == 192 and Display.GH == 256)

  local w = World.new(assets, Audio, 50000)
  check("new world has 5 lives", w.lives == 5)
  check("new world has 5 ownership bombs", w.bombs == 5)
  check("shot rank starts at TRAIT", w.power == 2)
  check("starts with no ai agents", #(w.agents or {}) == 0)

  w:recruitAgent({ id = "claude" })
  check("C pickup is yellow claude", w.agents[1] and w.agents[1].id == "claude")
  check("claude has tokens", w.agents[1] and (w.agents[1].tokens or 0) > 10)
  check("claude recruit banner", w.recruitFX and w.recruitFX.name == "CLAUDE")
  w:recruitAgent({ id = "grok" })
  check("G pickup is red grok", w.agents[2] and w.agents[2].id == "grok")
  w:recruitAgent({ id = "codex" })
  check("X pickup is green codex", w.agents[3] and w.agents[3].id == "codex")
  local nAgents = #w.agents
  w.agents[1].tokens = 3
  w:recruitAgent({ id = "claude" })
  check("same agent refills tokens", #w.agents == nAgents and w.agents[1].tokens > 10)
  w.agents[1].tokens = 0
  w:updateAgents(0.05)
  check("tokens empty agent leaves", not w:hasAgent("claude"))
  check("expired token sits on map", w.tokenItems[1] and w.tokenItems[1].tag == "C")
  w:dropItem(40, 40, "A")
  local last = w.tokenItems[#w.tokenItems]
  check("A drop becomes map token", last and (last.tag == "C" or last.tag == "G" or last.tag == "X"))

  local before = w.bombs
  local okBomb = w:doBomb()
  check("ownership bomb fires", okBomb == true and w.bombs == before - 1)
  check("ownership ring spawns", w.ownRing ~= nil)

  w:pshot(96, 200, 0, -300, 1, { col = { 0.4, 0.9, 1 } })
  check("player shot spawned", #w.pbullets >= 1)
  w:pshot(96, 200, 0, -220, 1.5, { homing = true, missile = true })
  check("tokio missile spawned", w.pbullets[#w.pbullets].missile == true)

  local e = w:spawn("beetle", 96, 40, { path = "down" })
  check("bug npc spawns", e ~= nil and e.kind == "beetle")
  local boss = w:spawn("boss", 96, -40, { path = "boss" })
  check("boss is segmentation fault", boss.boss == true and boss.img == "king")
  local ov = w:spawn("bossOverflow", 96, -40, { path = "boss" })
  check("stack overflow boss", ov.boss == true and ov.img == "bossOverflow")
  local dl = w:spawn("bossDeadlock", 96, -40, { path = "boss" })
  check("thread deadlock boss", dl.boss == true and dl.img == "bossDeadlock")

  w:spawnProp("cloud", 96, -20)
  check("small cloud scenery spawns", #w.scenery >= 1)

  local s1 = Stage.build(1)
  local s2 = Stage.build(2)
  local s3 = Stage.build(3)
  check("stage 1 has waves", type(s1) == "table" and #s1 >= 10)
  check("stage 2 has waves", type(s2) == "table" and #s2 >= 10)
  check("stage 3 has waves", type(s3) == "table" and #s3 >= 10)
  check("three stage names", Stage.NAMES[3] == "HKU CAMPUS")

  w.stage = 1
  w:advanceStage()
  check("advance to mtr stage", w.stage == 2)

  local kinds = {
    "offby1", "clippy", "lifetime", "infloop", "panic",
    "nullptr", "leak", "overflow", "deadlock",
  }
  local allOk = true
  for _, k in ipairs(kinds) do
    local ok = pcall(function()
      w:spawn(k, 10, -10, { path = "down" })
    end)
    if not ok then
      allOk = false
    end
  end
  check("coding-bug npcs spawn", allOk)

  local w3 = World.new(assets, Audio, 50000, 3)
  check("world can start at stage 3", w3.stage == 3)

  local Save = require "src.save"
  local rec = { event = "clear", stage = 2, score = 12345, tag = "1-2" }
  local line = Save.encode(rec)
  local back = Save.decode(line)
  check("jsonl encode is one object", type(line) == "string" and line:sub(1, 1) == "{")
  check("jsonl decode roundtrip", back and back.event == "clear" and back.stage == 2 and back.score == 12345)

  local dir = (love.filesystem.getSource() or ".") .. "/.tmp_save_test"
  Save.setDir(dir)
  os.remove(Save.path())
  Save.hiscore(88888)
  Save.clear(1, 12000)
  Save.map(3)
  Save.play(1)
  local prog = Save.load()
  check("jsonl persist hiscore", prog.hiscore >= 88888)
  check("jsonl persist clear", prog.cleared[1] == true)
  check("jsonl persist map cursor", prog.cursor == 3)
  check("jsonl persist play count", prog.plays >= 1)
  check("jsonl path is progress.jsonl", Save.path():match("progress%.jsonl$") ~= nil)
  os.remove(Save.path())
  Save.setDir(nil)

  print("1.." .. n)
  if fails == 0 then
    print("# all tests passed")
    return true
  end
  print("# " .. tostring(fails) .. " failed")
  return false
end

return M
