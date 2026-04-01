x = [
  1,
  2,
  3
]
y = [
  4,
  5
]
z = [
  6,
  7
]
# Closing bracket on own line with wrong indentation inside method call parens
foo([
      :bar,
      :baz
    ])
# FN fix: Splat *[ should still use paren-relative
List.new(:BULLET, *[
           ListItem.new(nil, Paragraph.new('l1')),
  ListItem.new(nil, Paragraph.new('l2'))
         ])
# FN fix: Single-pair hash should use line-relative, not hash-key-relative
requires_login except: [
  :index,
                 :show
]
# FN fix: String containing / should use paren-relative
Page.of_raw_data(site, '/', [
                   { name: "products" },
  { name: "categories" }
                 ])
# FN fix: Single-pair hash value in paren-relative — element + closing bracket at wrong indent
FactoryBot.create(:limited_admin, :groups => [
                    FactoryBot.create(:google_admin_group),
                  ])
# FN fix: Single-pair hash value in assert_equal — closing bracket at wrong indent
assert_equal({ "c" => [
               { "v" => 1421218800000, "f" => "Wed, Jan 14, 2015" },
  { "v" => 2, "f" => "2" },
             ] }, data["hits_over_time"]["rows"][1])
# FN fix: Empty array with wrong closing bracket indent
a << [
]
