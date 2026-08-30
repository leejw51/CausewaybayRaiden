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

  local w = World.new(assets, Audio, 50000, 1, "easy")
  check("easy kit has 5 lives", w.lives == 5)
  check("easy kit has 5 ownership bombs", w.bombs == 5)
  check("easy shot rank starts at TRAIT", w.power == 2)
  check("starts with no ai agents", #(w.agents or {}) == 0)
  check("easy rank id", w.difficulty == "easy")

  w:recruitAgent({ id = "claude" })
  check("C pickup is yellow claude", w.agents[1] and w.agents[1].id == "claude")
  check("claude has tokens", w.agents[1] and (w.agents[1].tokens or 0) > 10)
  check("easy agents auto-fire", w.agentAutoFire == true)
  check("claude eat radius is a shield", w.agents[1] and (w.agents[1].eatR or 0) >= 28)
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

  -- The save directory, not the source directory. `getSource()` is the .love
  -- archive itself once the game is packaged, and Save writes with plain
  -- io.open, so a path inside a zip is a path nothing can be created at --
  -- which is how these four checks passed from a checkout and failed from a
  -- bundle. getSaveDirectory() is a real writable path in both cases.
  local dir = (love.filesystem.getSaveDirectory() or ".") .. "/.tmp_save_test"
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

  local Balance = require "src.balance"
  check("three ranks", #Balance.LEVELS == 3
    and Balance.LEVELS[1] == "easy"
    and Balance.LEVELS[2] == "normal"
    and Balance.LEVELS[3] == "hard")
  check("default rank is normal", Balance.DEFAULT == "normal")
  check("bad rank falls back to normal", Balance.normalize("NOPE") == "normal")
  check("numeric rank 1 is easy", Balance.normalize(1) == "easy")
  check("numeric rank 3 is hard", Balance.normalize(3) == "hard")
  check("label is uppercase", Balance.label("hard") == "HARD")

  local be = Balance.get("easy")
  local bn = Balance.get("normal")
  local bh = Balance.get("hard")
  check("hard hp mul > normal", bh.hpMul > bn.hpMul)
  check("normal hp mul > easy", bn.hpMul > be.hpMul)
  check("hard fire > normal fire", bh.fireRateMul > bn.fireRateMul)
  check("normal fire > easy fire", bn.fireRateMul > be.fireRateMul)
  check("hard bullets faster than easy", bh.bulletSpdMul > be.bulletSpdMul)
  check("hard boss mul > easy boss mul", bh.bossHpMul > be.bossHpMul)
  check("harder ranks grant fewer lives", bh.lives < bn.lives and bn.lives <= be.lives)
  check("harder ranks grant fewer bombs", bh.bombs < bn.bombs and bn.bombs <= be.bombs)
  check("agents deal less on harder ranks", bh.agentDmgMul < bn.agentDmgMul and bn.agentDmgMul < be.agentDmgMul)
  check("agents shoot slower on harder ranks", bh.agentRateMul > bn.agentRateMul)
  check("easy agents fire faster than normal", be.agentRateMul < bn.agentRateMul)
  check("easy agents auto-fire flag", be.agentAutoFire == true)
  check("normal agents do not auto-spray", bn.agentAutoFire == false and bh.agentAutoFire == false)
  check("agents eat wider on harder ranks", bh.agentEatMul > bn.agentEatMul)
  check("normal agents prefer defense", bn.agentDmgMul < 0.5 and bn.agentEatMul >= 1.2)
  check("token rain is stingier on hard", bh.tokenRainEmpty > bn.tokenRainEmpty)

  local we = World.new(assets, Audio, 50000, 1, "easy")
  local wn = World.new(assets, Audio, 50000, 1, "normal")
  local wh = World.new(assets, Audio, 50000, 1, "hard")
  local wd = World.new(assets, Audio, 50000)
  check("omitted rank is normal", wd.difficulty == "normal" and wd.lives == 3 and wd.power == 1)
  check("normal kit is 3 lives 3 bombs", wn.lives == 3 and wn.bombs == 3 and wn.power == 1)
  check("hard kit is 2 lives 2 bombs", wh.lives == 2 and wh.bombs == 2 and wh.power == 1)
  check("world fire rate follows rank", wh.fireRateMul > wn.fireRateMul and wn.fireRateMul > we.fireRateMul)
  check("world hp mul follows rank", wh.hpMul > wn.hpMul and wn.hpMul > we.hpMul)
  check("easy world auto-fires agents", we.agentAutoFire == true)
  check("normal world agents stay defensive", wn.agentAutoFire == false)

  local mothE = we:spawn("moth", 40, -10, { path = "down" })
  local mothN = wn:spawn("moth", 40, -10, { path = "down" })
  local mothH = wh:spawn("moth", 40, -10, { path = "down" })
  check("fodder hp scales with rank", mothH.hp > mothN.hp and mothN.hp >= mothE.hp)

  local bossE = we:spawn("boss", 96, -40, { path = "boss" })
  local bossN = wn:spawn("boss", 96, 80, { path = "boss" })
  local bossH = wh:spawn("boss", 96, -40, { path = "boss" })
  check("normal boss has real hp", bossN.maxhp >= 600)
  check("hard boss has more hp than normal", bossH.hp > bossN.hp)
  check("normal boss has more hp than easy", bossN.hp > bossE.hp)
  check("boss hp is tracked as maxhp", bossN.hp == bossN.maxhp)
  local ovN = wn:spawn("bossOverflow", 96, -40, { path = "boss" })
  local dlN = wn:spawn("bossDeadlock", 96, -40, { path = "boss" })
  check("stage bosses are tanky", ovN.maxhp >= 350 and dlN.maxhp > ovN.maxhp and bossN.maxhp > dlN.maxhp)

  wn.readyT = 0
  wn.player.inv = 99
  wn.player.x, wn.player.y = 96, 220
  local hp0 = bossN.hp
  wn:pshot(bossN.x, bossN.y, 0, 0, 18)
  local idle = { left = false, right = false, up = false, down = false, shoot = false, bomb = false }
  wn:update(0.016, idle)
  check("player shot damages boss", bossN.hp < hp0 and bossN.hp > 0)
  check("boss flashes hp when hit", (bossN.hpFlash or 0) > 0)
  check("boss shows hp after hit", (bossN.hpShow or 0) > 0)
  check("one volley does not kill boss", bossN.dead ~= true and bossN.hp / bossN.maxhp > 0.9)
  check("boss keeps maxhp after hit", bossN.maxhp == hp0)

  we.agents = {}
  we.ebullets = {}
  we.readyT = 0
  we:recruitAgent({ id = "grok", quiet = true })
  local grokE = we:getAgent("grok")
  local easyShots = #we.pbullets
  we:agentShoot(grokE)
  check("easy grok auto-sprays a burst", #we.pbullets - easyShots >= 5)
  we.ebullets = { { x = grokE.x, y = grokE.y, vx = 0, vy = 20, r = 3, life = 4, grazed = false } }
  check("easy agent still shoots while blocking", we:agentShouldShoot(grokE) == true)
  grokE.fireT = 0
  local nShot = #we.pbullets
  we:updateAgents(0.05)
  check("easy agent fires without player input", #we.pbullets > nShot)

  wn.agents = {}
  wn.ebullets = {}
  wn:recruitAgent({ id = "grok", quiet = true })
  local grok = wn:getAgent("grok")
  check("grok eat radius is a shield", grok and (grok.eatR or 0) >= 36)
  check("grok fire interval is slow", grok and grok.rate >= 1.0)
  local shotsBefore = #wn.pbullets
  wn:agentShoot(grok)
  check("grok no longer sprays a burst", #wn.pbullets - shotsBefore <= 2)
  wn.ebullets = { { x = grok.x, y = grok.y, vx = 0, vy = 20, r = 3, life = 4, grazed = false } }
  check("agent holds fire to block bullets", wn:agentShouldShoot(grok) == false)
  check("agent eats a covering bullet", wn:tryAgentEat(grok.x, grok.y) == true)
  wn.ebullets = {}
  check("agent may shoot when sky is clear", wn:agentShouldShoot(grok) == true)
  local threat = { x = wn.player.x + 8, y = wn.player.y - 20, vx = 0, vy = 40, r = 3, life = 4, grazed = false }
  wn.ebullets = { threat }
  grok.x, grok.y = wn.player.x - 30, wn.player.y
  local near = wn:nearestThreatBullet(grok, 86)
  check("agent intercepts bullets near the ship", near == threat)

  local dpsN = (1.1 * 1.35) / 0.07
  check("normal boss outlasts a few seconds of PRINTLN", bossN.maxhp / dpsN >= 12)

  print("1.." .. n)
  if fails == 0 then
    print("# all tests passed")
    return true
  end
  print("# " .. tostring(fails) .. " failed")
  return false
end

return M
