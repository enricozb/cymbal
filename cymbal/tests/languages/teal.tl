-- Functions
local function x()
end
global function y()
end
local x = function() end
global y = function() end

-- Methods
function Point.new()
end
function Point:move()
end

-- Structs
local record Obj
end
global record Obj
end

-- Interfaces
local interface MyAbstractInterface
end
global interface MyAbstractInterface
end

-- Types
local type Enum = enum
end
local type Record = record
end

-- Globals
global x
global y

-- Macros
local macroexp add()
  return
end
