if cond
  foo
else
end

if x > 1
  bar
else
end

case status
when :active
  activate
else
end

# elsif without final else — offense on the last elsif
if x > 1
  bar
elsif x < 0
  baz
else
end

# Multiple elsif without final else — offense on LAST elsif only
if cond_1
  one
elsif cond_2
  two
elsif cond_3
  three
else
end

while ppid != '1'
  if (agent = agents[ppid])
    ENV['SSH_AUTH_SOCK'] = agent
    break
  else
  end
  File.open("/proc/#{ppid}/status", "r") do |file|
    ppid = file.read().match(/PPid:\s+(\d+)/)[1]
  end
end

def restart_service(service_name)
  service_running = run_context.resource_collection.lookup("service[#{service_name}]") rescue nil

  if service_running
    execute "trigger-notify-restart-#{service_name}" do
      command "true"
      notifies :restart, "service[#{service_name}]"
      only_if "true"
    end
  else
  end
end

if node['ariadne']['clean']
  project = node['ariadne']['project']

  execute "chmod -R 777 /mnt/www/html/#{project}" do
    only_if "test -d /mnt/www/html/#{project}"
  end

  %W{
    /vagrant/data/profiles/#{project}
    /mnt/www/html/#{project}
  }.each do |dir|
    directory dir do
      recursive true
      action :delete
      only_if "test -d #{dir}"
    end
  end
else
end

if node['instance_role'] == 'vagrant'
  ::Chef::Resource::RubyBlock.send(:include, Ariadne::Helpers)
  ruby_block "Give root access to the forwarded ssh agent" do
    block do
      give_ssh_agent_root
    end
  end
else
end

case node['platform']
when "debian", "ubuntu"
  package "php5-memcached" do
    action :install
  end
else
end
