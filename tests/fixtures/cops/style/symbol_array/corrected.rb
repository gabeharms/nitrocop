%i[foo bar baz]

%i[one two]

x = %i[alpha beta gamma delta]

# Symbol arrays inside block body of non-parenthesized call should still be flagged
# (only direct arguments are ambiguous, not nested arrays in block body)
describe "test" do
  %i[admin read write]
end

it "works" do
  x = %i[foo bar]
end

context "scope" do
  let(:roles) do
    %i[viewer editor]
  end
end

# Symbol arrays inside keyword args of ambiguous calls — not truly ambiguous,
# RuboCop only suppresses top-level (bare) arguments, not hash values
resources :posts, only: %i[index show] do
  member do
    get :preview
  end
end
