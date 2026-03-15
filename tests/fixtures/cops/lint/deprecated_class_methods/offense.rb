File.exists?("foo")
^^^^^^^^^^^^^^^^^^^ Lint/DeprecatedClassMethods: `File.exists?` is deprecated in favor of `File.exist?`.
Dir.exists?("bar")
^^^^^^^^^^^^^^^^^^ Lint/DeprecatedClassMethods: `Dir.exists?` is deprecated in favor of `Dir.exist?`.
File.exists?("baz")
^^^^^^^^^^^^^^^^^^^ Lint/DeprecatedClassMethods: `File.exists?` is deprecated in favor of `File.exist?`.
::File.exists?("qux")
^^^^^^^^^^^^^^^^^^^^^ Lint/DeprecatedClassMethods: `::File.exists?` is deprecated in favor of `::File.exist?`.
::Dir.exists?("quux")
^^^^^^^^^^^^^^^^^^^^^ Lint/DeprecatedClassMethods: `::Dir.exists?` is deprecated in favor of `::Dir.exist?`.
ENV.clone
^^^^^^^^^ Lint/DeprecatedClassMethods: `ENV.clone` is deprecated in favor of `ENV.to_h`.
ENV.dup
^^^^^^^ Lint/DeprecatedClassMethods: `ENV.dup` is deprecated in favor of `ENV.to_h`.
ENV.freeze
^^^^^^^^^^ Lint/DeprecatedClassMethods: `ENV.freeze` is deprecated in favor of `ENV`.
iterator?
^^^^^^^^^ Lint/DeprecatedClassMethods: `iterator?` is deprecated in favor of `block_given?`.
attr :name, true
^^^^^^^^^^^^^^^^ Lint/DeprecatedClassMethods: `attr :name, true` is deprecated in favor of `attr_accessor :name`.
attr :name, false
^^^^^^^^^^^^^^^^^ Lint/DeprecatedClassMethods: `attr :name, false` is deprecated in favor of `attr_reader :name`.
attr 'title', true
^^^^^^^^^^^^^^^^^^ Lint/DeprecatedClassMethods: `attr 'title', true` is deprecated in favor of `attr_accessor 'title'`.
Socket.gethostbyname("hal")
^^^^^^^^^^^^^^^^^^^^^^^^^^^ Lint/DeprecatedClassMethods: `Socket.gethostbyname` is deprecated in favor of `Addrinfo.getaddrinfo`.
Socket.gethostbyaddr([221,186,184,68].pack("CCCC"))
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Lint/DeprecatedClassMethods: `Socket.gethostbyaddr` is deprecated in favor of `Addrinfo#getnameinfo`.
::Socket.gethostbyname("hal")
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Lint/DeprecatedClassMethods: `::Socket.gethostbyname` is deprecated in favor of `Addrinfo.getaddrinfo`.
