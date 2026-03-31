alias :bar :foo

alias :new_name :old_name

alias :greet :hello

class C
  alias :ala :bala
end

module M
  alias :ala :bala
end

# alias inside class_eval block should use alias_method (dynamic scope)
SomeClass.class_eval do
  alias_method :new_name, :old_name
end

# alias inside module_eval block should use alias_method (dynamic scope)
SomeModule.module_eval do
  alias_method :new_name, :old_name
end
