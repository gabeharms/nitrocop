OpenSSL::Cipher.new('aes-128-gcm')
OpenSSL::Digest.new('SHA256')
OpenSSL::Digest.digest('SHA256', 'foo')
cipher = OpenSSL::Cipher.new('des')
digest = OpenSSL::Digest.new('MD5')
OpenSSL::Cipher.new(algorithm)

# Dynamic arguments: skip when args contain variables, calls, or constants
OpenSSL::Digest::SHA256.digest(data)
OpenSSL::Digest::SHA256.digest(some_method_call)
OpenSSL::Digest::SHA256.new(cert.to_der)
OpenSSL::Cipher::AES.new(key_length, mode)
OpenSSL::Digest::SHA1.new(cert.to_der)

# OpenSSL::Digest::Digest is an old alias for OpenSSL::Digest itself, not an algorithm constant
OpenSSL::Digest::Digest.new('sha1')
DIGEST = OpenSSL::Digest::Digest.new('sha1')
OpenSSL::Digest::Digest.new('md5')
OpenSSL::Digest::Digest.new('sha256')
