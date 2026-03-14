x.blank?
x.present?
!x.empty?
x.nil?
name.present? && name.length > 0
x.nil? || y.empty?
x.nil? && x.empty?
x.nil? || x.zero?
something if foo.present?
something unless foo.blank?
def blank?
  !present?
end
unless foo.present?
  something
else
  something_else
end

# !foo is a boolean negation, not a nil check — should not trigger NilOrEmpty
!foo || foo.empty?
!record || record.empty?
!@item || @item.empty?

# present? called with argument (class method style) should NOT be flagged
# RuboCop's NodePattern `(send (send $_ :present?) :!)` requires present? with no arguments
!Helpers.present?(value)
!Vagrant::Util::Presence.present?(directory)
unless Helpers.present?(value)
  do_something
end
