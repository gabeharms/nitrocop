x = 'this text is too ' \
    'long'

y = 'this text contains a lot of               ' \
    'spaces'

z = "another example " \
    "with leading space"

error = "go: example.com/tool@v1.0.0 requires\n	" \
    "github.com/example/dependency@v0.0.0-00010101000000-000000000000: invalid version"

mixed = "foo #{bar} " \
  'long'

logger.warn("Downcasing dependency '#{name}' because deb packages  " \
             "don't work so good with uppercase names")

msg = "expected #{resource} to have  " \
  "the correct value"

hint = "Use #{method_name} instead of   " \
  "calling directly"

message = %Q{expected "#{resource}" to have parameters:} \
  "\n\n  " \
  "" + unmatched.collect { |p, h| p }

raise SpoofError, "IP spoofing attack?! " \
  "HTTP_CLIENT_IP=#{req.client_ip} " \
  "HTTP_X_FORWARDED_FOR=#{req.forwarded_for} " \
  "HTTP_FORWARDED=" + req.forwarded.map { "for=#{_1}" }.join(", ")

