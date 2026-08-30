-- Persist game progress as JSONL in ~/.causewayraiden

local Save = {}

local function homeDir()
  return os.getenv("HOME") or os.getenv("USERPROFILE") or "."
end

function Save.dir()
  return (Save._dirOverride or (homeDir() .. "/.causewayraiden"))
end

function Save.setDir(path)
  Save._dirOverride = path
end

function Save.path()
  return Save.dir() .. "/progress.jsonl"
end

local function ensureDir()
  local d = Save.dir()
  local ok = io.open(d, "r")
  if ok then
    ok:close()
    return
  end
  os.execute('mkdir -p "' .. d:gsub('"', '\\"') .. '"')
end

function Save.encode(v)
  local t = type(v)
  if v == nil then
    return "null"
  elseif t == "boolean" then
    return v and "true" or "false"
  elseif t == "number" then
    if v ~= v or v == math.huge or v == -math.huge then
      return "0"
    end
    return tostring(v)
  elseif t == "string" then
    return '"' .. v:gsub("\\", "\\\\"):gsub('"', '\\"'):gsub("\n", "\\n"):gsub("\r", "\\r") .. '"'
  elseif t == "table" then
    local n = #v
    local isArr = n > 0
    if isArr then
      for i = 1, n do
        if v[i] == nil then
          isArr = false
          break
        end
      end
    end
    if isArr then
      local parts = {}
      for i = 1, n do
        parts[i] = Save.encode(v[i])
      end
      return "[" .. table.concat(parts, ",") .. "]"
    end
    local keys = {}
    for k in pairs(v) do
      if type(k) == "string" then
        keys[#keys + 1] = k
      end
    end
    table.sort(keys)
    local parts = {}
    for _, k in ipairs(keys) do
      parts[#parts + 1] = Save.encode(k) .. ":" .. Save.encode(v[k])
    end
    return "{" .. table.concat(parts, ",") .. "}"
  end
  return "null"
end

local function skip(s, i)
  while i <= #s do
    local c = s:sub(i, i)
    if c ~= " " and c ~= "\t" then
      return i
    end
    i = i + 1
  end
  return i
end

local parseValue

local function parseString(s, i)
  i = i + 1
  local out = {}
  while i <= #s do
    local c = s:sub(i, i)
    if c == '"' then
      return table.concat(out), i + 1
    elseif c == "\\" then
      local n = s:sub(i + 1, i + 1)
      if n == "n" then
        out[#out + 1] = "\n"
      elseif n == "r" then
        out[#out + 1] = "\r"
      else
        out[#out + 1] = n
      end
      i = i + 2
    else
      out[#out + 1] = c
      i = i + 1
    end
  end
  return table.concat(out), i
end

local function parseNumber(s, i)
  local j = i
  if s:sub(i, i) == "-" then
    i = i + 1
  end
  while s:sub(i, i):match("%d") do
    i = i + 1
  end
  if s:sub(i, i) == "." then
    i = i + 1
    while s:sub(i, i):match("%d") do
      i = i + 1
    end
  end
  return tonumber(s:sub(j, i - 1)) or 0, i
end

local function parseObject(s, i)
  local obj = {}
  i = skip(s, i + 1)
  if s:sub(i, i) == "}" then
    return obj, i + 1
  end
  while i <= #s do
    i = skip(s, i)
    local key
    key, i = parseString(s, i)
    i = skip(s, i)
    if s:sub(i, i) ~= ":" then
      break
    end
    i = skip(s, i + 1)
    local val
    val, i = parseValue(s, i)
    obj[key] = val
    i = skip(s, i)
    local c = s:sub(i, i)
    if c == "}" then
      return obj, i + 1
    elseif c == "," then
      i = i + 1
    else
      return obj, i
    end
  end
  return obj, i
end

local function parseArray(s, i)
  local arr = {}
  i = skip(s, i + 1)
  if s:sub(i, i) == "]" then
    return arr, i + 1
  end
  while i <= #s do
    i = skip(s, i)
    local val
    val, i = parseValue(s, i)
    arr[#arr + 1] = val
    i = skip(s, i)
    local c = s:sub(i, i)
    if c == "]" then
      return arr, i + 1
    elseif c == "," then
      i = i + 1
    else
      return arr, i
    end
  end
  return arr, i
end

parseValue = function(s, i)
  i = skip(s, i)
  local c = s:sub(i, i)
  if c == '"' then
    return parseString(s, i)
  elseif c == "{" then
    return parseObject(s, i)
  elseif c == "[" then
    return parseArray(s, i)
  elseif c == "t" and s:sub(i, i + 3) == "true" then
    return true, i + 4
  elseif c == "f" and s:sub(i, i + 4) == "false" then
    return false, i + 5
  elseif c == "n" and s:sub(i, i + 3) == "null" then
    return nil, i + 4
  else
    return parseNumber(s, i)
  end
end

function Save.decode(line)
  if not line or line == "" then
    return nil
  end
  local ok, val = pcall(parseValue, line, 1)
  if ok and type(val) == "table" then
    return val
  end
  return nil
end

function Save.append(rec)
  rec.ts = rec.ts or os.time()
  ensureDir()
  local f = io.open(Save.path(), "a")
  if not f then
    return false
  end
  f:write(Save.encode(rec) .. "\n")
  f:close()
  return true
end

function Save.load()
  local prog = {
    hiscore = 50000,
    cleared = {},
    cursor = 1,
    plays = 0,
  }
  local f = io.open(Save.path(), "r")
  if not f then
    return prog
  end
  for line in f:lines() do
    local rec = Save.decode(line)
    if rec and rec.event then
      if rec.event == "hiscore" and type(rec.value) == "number" then
        prog.hiscore = math.max(prog.hiscore, rec.value)
      elseif rec.event == "clear" and type(rec.stage) == "number" then
        prog.cleared[rec.stage] = true
        if type(rec.score) == "number" then
          prog.hiscore = math.max(prog.hiscore, rec.score)
        end
        prog.cursor = math.max(prog.cursor, math.min(3, rec.stage + 1))
      elseif rec.event == "map" and type(rec.cursor) == "number" then
        prog.cursor = math.max(1, math.min(3, rec.cursor))
      elseif rec.event == "play" then
        prog.plays = prog.plays + 1
      end
    end
  end
  f:close()
  return prog
end

function Save.hiscore(n)
  Save.append({ event = "hiscore", value = math.floor(n) })
end

function Save.clear(stage, score)
  Save.append({ event = "clear", stage = stage, score = math.floor(score or 0) })
end

function Save.map(cursor)
  Save.append({ event = "map", cursor = cursor })
end

function Save.play(stage)
  Save.append({ event = "play", stage = stage })
end

return Save
