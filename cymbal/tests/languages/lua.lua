-- Functions
local function x() end
global function y() end
local x = function() end
global y = function() end

-- Method
function Point.new() end
function Point:move() end

-- Global
global x = ""
global y = {}
global z
