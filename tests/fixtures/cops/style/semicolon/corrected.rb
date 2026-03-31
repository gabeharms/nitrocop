x = 1; y = 2

a = 1; b = 2; c = 3

foo; bar

def guard; log('guard'); !@fail_guard; end

def foo(a) x(1); y(2); z(3); end

foo { bar }

items.each { bar }

arr.map { baz }

"#{foo}"

x = "#{foo}"

"prefix #{foo}"

"#{foo}"

x = "a;b"; y = 2

def prx; end
def r500(*); end
module X
  def self.D(*); end
end

def call e
  k,m,*a=X.D e["PATH_INFO"],e['REQUEST_METHOD'].
  downcase,e;k.new(e,m,prx).service(*a).to_a;rescue;r500(:I,k,m,$!,:env=>e).to_a
end

@@parameters = {}
@@aliases = {}
@@arity = {}
@@defaults = {
  parameters: @@parameters.each_with_object({}) { |(k, v), p| p[k] = v.dup },
  aliases: @@aliases.dup,
  arity: @@arity.dup
}
