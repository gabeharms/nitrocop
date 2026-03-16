Rails.logger.debug { "The time is #{Time.zone.now}." }
Rails.logger.debug "Simple string without interpolation"
Rails.logger.info "Info: #{user.name}"
Rails.logger.debug "plain message"
logger.debug "not Rails.logger"
puts "not a logger call"
Rails.logger&.debug("Could not auto-detect path: #{e.message}")
Rails.logger&.debug "Safe nav interpolation: #{value}"

# Sole statement in block body — RuboCop's `node.parent&.block_type?` skips this
call.on_success do
  Rails.logger.debug "[LDAP groups] Added users #{user_ids} to #{group.name}"
end

records.each do |record|
  Rails.logger.debug("Deleting record: #{record.slice(:id, :name)}")
end

# Nested: inner block with a SINGLE debug statement — both blocks are sole-stmt
# The inner block's parent IS block_type?, so still no offense.
items.each do |item|
  Post.transaction do
    Rails.logger.debug "Single stmt in inner block: #{item.name}"
  end
end
