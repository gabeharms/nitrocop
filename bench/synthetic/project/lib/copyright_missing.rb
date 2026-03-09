# This file intentionally has no copyright notice.
# It should trigger Style/Copyright.

class CopyrightMissing
  def initialize
    @name = "test"
  end

  def call
    puts @name
  end
end
