# nitrocop-filename: spec/models/user_spec.rb
RSpec.describe User, type: :model do
                     ^^^^^^^^^^^^ RSpecRails/InferredSpecType: Remove redundant spec type.
end

describe User, type: :model do
               ^^^^^^^^^^^^ RSpecRails/InferredSpecType: Remove redundant spec type.
end

RSpec.describe User, other: true, type: :model do
                                  ^^^^^^^^^^^^ RSpecRails/InferredSpecType: Remove redundant spec type.
end

xdescribe User, type: :model do
                ^^^^^^^^^^^^ RSpecRails/InferredSpecType: Remove redundant spec type.
end

# No described class — type still redundant when it matches inferred type
RSpec.describe type: :model do
               ^^^^^^^^^^^^ RSpecRails/InferredSpecType: Remove redundant spec type.
end

describe type: :model do
         ^^^^^^^^^^^^ RSpecRails/InferredSpecType: Remove redundant spec type.
end

# No described class with additional metadata
RSpec.describe type: :model, swars_spec: true do
               ^^^^^^^^^^^^ RSpecRails/InferredSpecType: Remove redundant spec type.
end
