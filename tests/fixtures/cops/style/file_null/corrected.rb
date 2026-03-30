path = File::NULL
CONST = File::NULL
path = File::NULL

:Logger => ENV['WEBRICK_DEBUG'].nil? ? WEBrick::Log.new(File::NULL) : nil,

Logger: WEBrick::Log.new(File::NULL),

@cache = Memcached::Rails.new(:servers => @servers, :namespace => @namespace, :logger => Logger.new(File.open(File::NULL, "w")))

Logger: WEBrick::Log.new(File::NULL),

server = WEBrick::GenericServer.new(Port: 0, Logger: Logger.new(File::NULL))

:logger => Logger.new(File::NULL),

:logger => Logger.new(File::NULL),

:logger => Logger.new(File::NULL),
