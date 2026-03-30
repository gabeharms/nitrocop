'foo'.unpack1('h*')

'foo'.unpack1('h*')

'foo'.unpack1('h*')

OpenSSL::PKCS5.pbkdf2_hmac(
  mnemonic, salt, 2048, 64, OpenSSL::Digest::SHA512.new
).unpack1('H*')

OpenSSL::PKCS5.pbkdf2_hmac(
  password,
  salt,
  iterations,
  128,
  OpenSSL::Digest.new("SHA512")
).unpack1("H*")
