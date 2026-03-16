x =
  1

y = 2

CONST =
  "hello"

# Conditional assignments - RuboCop does not flag these
if upload =
     Upload.find_by(name: params[:name])
  process(upload)
end

if match =
     url.match(%r{/foo/bar})
  handle(match)
end

while item =
        queue.pop
  process(item)
end

# Same-line RHS is always OK
x = if condition
      1
    end

# Properly indented embedded assignment
def foo
  if result =
       compute(value)
    result
  end
end

# Properly indented operator assignments
x +=
  1

y ||=
  "default"

# Properly indented setter/index assignments
result[:key] =
  hash_from_xml(data)

self.name =
  compute_name(input)

obj.attr ||=
  default_value

# Multiline bracket LHS with value on same line as `=` — not a multi-line assignment
headers[
  "X-Custom-Header"
] = "some_value"

serializer_opts[:field_map][
  "#{prefix}#{field_id}"
] = field_id

config[
  key
] = value

obj.attributes[
  "name"
] ||= "default"

items[
  index
] += extra

# Chained assignments — inner write uses outermost assignment's indentation
@stroke_color = @fill_color =
  GlobalConfig.constantize('color_space.map', :DeviceGray).new.default_color

off_form = @widget.appearance_dict.normal_appearance[:Off] =
  @document.add({Type: :XObject, Subtype: :Form, BBox: [0, 0, width, height],
                 Matrix: matrix})

gem.description = gem.summary =
  'Process monitoring tool'

result = cache[key] ||=
  begin
    compute_value
  end

configs = app.config.active_storage.service_configurations ||=
  begin
    load_configurations
  end

rules = LOAD_RULES_CACHE[self.class.rules_cache_key] ||=
  self.class.files.each_with_object({}) do |filename, hash|
    hash[filename] = true
  end
