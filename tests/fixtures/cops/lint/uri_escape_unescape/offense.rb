URI.escape("http://example.com")
^ Lint/UriEscapeUnescape: `URI.escape` method is obsolete and should not be used. Instead, use `CGI.escape`, `URI.encode_www_form` or `URI.encode_www_form_component` depending on your specific use case.
URI.unescape("%20")
^ Lint/UriEscapeUnescape: `URI.unescape` method is obsolete and should not be used. Instead, use `CGI.unescape`, `URI.decode_www_form` or `URI.decode_www_form_component` depending on your specific use case.
URI.escape("another")
^ Lint/UriEscapeUnescape: `URI.escape` method is obsolete and should not be used. Instead, use `CGI.escape`, `URI.encode_www_form` or `URI.encode_www_form_component` depending on your specific use case.
::URI.escape("qualified")
^ Lint/UriEscapeUnescape: `::URI.escape` method is obsolete and should not be used. Instead, use `CGI.escape`, `URI.encode_www_form` or `URI.encode_www_form_component` depending on your specific use case.
::URI.unescape("qualified")
^ Lint/UriEscapeUnescape: `::URI.unescape` method is obsolete and should not be used. Instead, use `CGI.unescape`, `URI.decode_www_form` or `URI.decode_www_form_component` depending on your specific use case.
URI.encode("http://example.com")
^ Lint/UriEscapeUnescape: `URI.encode` method is obsolete and should not be used. Instead, use `CGI.escape`, `URI.encode_www_form` or `URI.encode_www_form_component` depending on your specific use case.
::URI.encode("http://example.com")
^ Lint/UriEscapeUnescape: `::URI.encode` method is obsolete and should not be used. Instead, use `CGI.escape`, `URI.encode_www_form` or `URI.encode_www_form_component` depending on your specific use case.
URI.decode(enc_uri)
^ Lint/UriEscapeUnescape: `URI.decode` method is obsolete and should not be used. Instead, use `CGI.unescape`, `URI.decode_www_form` or `URI.decode_www_form_component` depending on your specific use case.
::URI.decode(enc_uri)
^ Lint/UriEscapeUnescape: `::URI.decode` method is obsolete and should not be used. Instead, use `CGI.unescape`, `URI.decode_www_form` or `URI.decode_www_form_component` depending on your specific use case.
