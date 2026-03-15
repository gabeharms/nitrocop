!x.present?
^^^^^^^^^^^ Rails/Blank: Use `blank?` instead of `!present?`.

!name.present?
^^^^^^^^^^^^^^ Rails/Blank: Use `blank?` instead of `!present?`.

!user.email.present?
^^^^^^^^^^^^^^^^^^^^ Rails/Blank: Use `blank?` instead of `!present?`.

x.nil? || x.empty?
^^^^^^^^^^^^^^^^^^^ Rails/Blank: Use `x.blank?` instead of `x.nil? || x.empty?`.

name.nil? || name.empty?
^^^^^^^^^^^^^^^^^^^^^^^^ Rails/Blank: Use `name.blank?` instead of `name.nil? || name.empty?`.

foo == nil || foo.empty?
^^^^^^^^^^^^^^^^^^^^^^^^ Rails/Blank: Use `foo.blank?` instead of `foo == nil || foo.empty?`.

something unless foo.present?
          ^^^^^^^^^^^^^^^^^^^ Rails/Blank: Use `if foo.blank?` instead of `unless foo.present?`.

something unless present?
          ^^^^^^^^^^^^^^^ Rails/Blank: Use `if blank?` instead of `unless present?`.

unless foo.present?
^^^^^^^^^^^^^^^^^^^ Rails/Blank: Use `if foo.blank?` instead of `unless foo.present?`.
  something
end

!foo || foo.empty?
^^^^^^^^^^^^^^^^^^ Rails/Blank: Use `foo.blank?` instead of `!foo || foo.empty?`.

!methods || methods.empty?
^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/Blank: Use `methods.blank?` instead of `!methods || methods.empty?`.

!url || url.empty?
^^^^^^^^^^^^^^^^^^ Rails/Blank: Use `url.blank?` instead of `!url || url.empty?`.

return self if nil? || empty?
               ^^^^^^^^^^^^^^ Rails/Blank: Use `blank?` instead of `nil? || empty?`.
return [] if nil? || empty?
             ^^^^^^^^^^^^^^ Rails/Blank: Use `blank?` instead of `nil? || empty?`.
