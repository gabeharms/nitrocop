# Copyright 2025 Acme Inc.

require_relative 'require_self'

# RequireRelativeSelfPath: the require_relative above references this file itself.
class RequireSelf
  def run
    puts "loaded"
  end
end
