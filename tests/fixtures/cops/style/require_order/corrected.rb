require 'a'
require 'b'

require 'a'
require 'c'
require 'z'

require 'a'
require 'b'
require 'c'
require 'd'

require_relative 'b'
require_relative 'c'
require_relative 'd'
require_relative 'z'

require 'c'
require 'a' if foo
require 'b'

require 'b'
# comment
require 'a'

require 'b'
# require 'z'
require 'a'

require 'b'
# multiple
# comments
require 'a'

require("a")
require("b")

require_relative("a")
require_relative("b")

require 'rack'
require 'webmachine/adapter'
require 'webmachine/constants'
