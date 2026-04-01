def something
  x = Something.new
  x.attr = 5
  x
end

def another
  obj = Object.new
  obj.name = 'foo'
  obj
end

def third
  item = Item.new
  item.price = 100
  item
end

# Square bracket setter on local object
def bracket_setter
  data = {}
  data[:key] = 'value'
  data
end

# Variable from multi-assignment with non-array RHS (local object)
def multi_assign_local
  _first, target, _third = do_something
  target.attr = 5
  target
end

# Argument reassigned to local object — no longer the passed object
def reassigned_arg(arg)
  arg = Top.new
  arg.attr = 5
  arg
end

# Variable assigned via binary operator assignment (local object)
def binary_op_assign(arg)
  result = arg
  result += arg
  result.attr = 5
  result
end

# Variable assigned from literal (local object)
def literal_assign
  data = []
  data[:key] = 1
  data
end
