foo ||= begin
  bar
  baz
end

foo ||=
  begin
    bar
    baz
  end

foo ||= (bar ||
          baz)

@info["exif"] ||= begin
  hash = {}
  output = self["%[EXIF:*]"]
  hash
end

foo.bar ||= begin
  x = 1
  y = 2
end
