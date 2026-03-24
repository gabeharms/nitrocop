# nitrocop-filename: spec/models/user_spec.rb
RSpec.describe User do
end

RSpec.describe User, type: :common do
end

RSpec.describe User, type: :controller do
end

# No described class, type mismatches inferred type — not redundant
RSpec.describe type: :controller do
end

describe type: :controller do
end
