def func
  begin
    ala
  rescue => e
    bala
  end
end

def Test.func
  begin
    ala
  rescue => e
    bala
  end
end

def bar
  begin
    do_something
  ensure
    cleanup
  end
end

# Redundant begin in ||= assignment with single statement
@current_resource_owner ||= instance_eval(&Doorkeeper.config.authenticate_resource_owner)

# Redundant begin in = assignment with single statement
x = compute_value

# Redundant begin in local variable ||= assignment
value ||= calculate

# Redundant begin inside a do..end block
items.each do |item|
  begin
    process(item)
  rescue StandardError => e
    handle(e)
  end
end

# Redundant begin inside a lambda block
Thread.new do
  begin
    run_task
  rescue => e
    log(e)
  end
end

# Redundant begin inside a block with ensure
run do
  begin
    perform
  ensure
    cleanup
  end
end
