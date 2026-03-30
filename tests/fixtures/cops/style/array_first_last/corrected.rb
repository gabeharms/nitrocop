arr.first

arr.last

items.first

# Inside array literal that is argument to []=
hash[key] = [arr.first, records.last]

# Compound assignment on indexed access (IndexOperatorWriteNode)
padding[0] += delta

line_widths[-1] += width

options[0] += 1

# Logical-or assignment on indexed access (IndexOrWriteNode)
params[0] ||= "localhost"

colors[-1] ||= "red"

# Logical-and assignment on indexed access (IndexAndWriteNode)
items[0] &&= transform(value)

# Explicit method call syntax: arr.[](0)
arr.first

arr.last

# Safe-navigation explicit method call: arr&.[](0)
arr&.first

arr&.last

exif.first&.raw_fields&.[](BORDER_TAG_IDS[border])&.[](0)

assert_equal "hello", result.first.content[0][:text]

assert_equal "world", result.first.content[1][:text]

inner_doc = doc.blocks.first.rows.body[0][0].inner_document

cell = (document_from_string input).blocks.first.rows.body[0][0]

dd = doc.blocks.first.items[0][1]

result[pair.children.first.children[0]] = Solargraph::Parser.chain(pair.children[1])

credential[:tokentype] = tokentype.first.split(":")[1]
