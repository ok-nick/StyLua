local x
something((x))

local x = (1 + 2) * 3
local y = ((1) * 3)
local z = (...) == nil and foo or bar
local foo = not (bar and baz)
local bar = (not bar) and baz
local cond = condition and (not object or object.Value == y)
local baz = (-4 + 3) * 2

({}):foo();
("hello"):format()

function x()
	return 1, 2
end
  
print(x())
print((x()))
print(((x())))

path = (function()
  return true
end)()