path = '/dev/null'
       ^^^^^^^^^^^ Style/FileNull: Use `File::NULL` instead of `/dev/null`.
CONST = '/dev/null'
        ^^^^^^^^^^^ Style/FileNull: Use `File::NULL` instead of `/dev/null`.
path = "/dev/null"
       ^^^^^^^^^^^ Style/FileNull: Use `File::NULL` instead of `/dev/null`.
Logger.new("/dev/null")
           ^^^^^^^^^^^ Style/FileNull: Use `File::NULL` instead of `/dev/null`.
Server.new(Port: 0, Logger: Logger.new("/dev/null"))
                                       ^^^^^^^^^^^ Style/FileNull: Use `File::NULL` instead of `/dev/null`.
Klogger.new(nil, destination: enabled ? $stdout : "/dev/null", highlight: true)
                                                  ^^^^^^^^^^^ Style/FileNull: Use `File::NULL` instead of `/dev/null`.
