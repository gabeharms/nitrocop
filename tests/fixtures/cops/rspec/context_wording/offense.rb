context 'the display name not present' do
        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/ContextWording: Context description should match /^when\b/, /^with\b/, /^without\b/.
end

context 'whenever you do' do
        ^^^^^^^^^^^^^^^^^ RSpec/ContextWording: Context description should match /^when\b/, /^with\b/, /^without\b/.
end

shared_context 'the display name not present' do
               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/ContextWording: Context description should match /^when\b/, /^with\b/, /^without\b/.
end

# Interpolated string descriptions should also be checked
context "Fabricate(:#{fabricator_name})" do
        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/ContextWording: Context description should match /^when\b/, /^with\b/, /^without\b/.
end

# Backtick (xstr) descriptions should also be checked
context `the display name not present` do
        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/ContextWording: Context description should match /^when\b/, /^with\b/, /^without\b/.
end

context `bad #{interpolated} description` do
        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/ContextWording: Context description should match /^when\b/, /^with\b/, /^without\b/.
end
