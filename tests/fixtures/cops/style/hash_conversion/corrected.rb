ary.to_h
{a => b, c => d}
{}
{a: b, c: d}
result = (items.map do |k, v|
  [k, Hash[v.map { |x| [x, true] }]]
end).to_h
(records.map { |r| [r.id, Hash[r.attrs.map { |a| [a.name, a.value] }]] }).to_h
