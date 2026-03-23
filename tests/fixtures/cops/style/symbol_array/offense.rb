[:foo, :bar, :baz]
^ Style/SymbolArray: Use `%i` or `%I` for an array of symbols.

[:one, :two]
^ Style/SymbolArray: Use `%i` or `%I` for an array of symbols.

x = [:alpha, :beta, :gamma, :delta]
    ^ Style/SymbolArray: Use `%i` or `%I` for an array of symbols.

# Symbol arrays inside block body of non-parenthesized call should still be flagged
# (only direct arguments are ambiguous, not nested arrays in block body)
describe "test" do
  [:admin, :read, :write]
  ^ Style/SymbolArray: Use `%i` or `%I` for an array of symbols.
end

it "works" do
  x = [:foo, :bar]
      ^ Style/SymbolArray: Use `%i` or `%I` for an array of symbols.
end

context "scope" do
  let(:roles) do
    [:viewer, :editor]
    ^ Style/SymbolArray: Use `%i` or `%I` for an array of symbols.
  end
end

# Symbol arrays inside keyword args of ambiguous calls — not truly ambiguous,
# RuboCop only suppresses top-level (bare) arguments, not hash values
resources :posts, only: [:index, :show] do
                        ^ Style/SymbolArray: Use `%i` or `%I` for an array of symbols.
  member do
    get :preview
  end
end

# Block-pass (&block) is NOT an ambiguous block context — only literal do/end or {} blocks are
@recorder.inverse_of :drop_table, [:musics, :artists], &block
                                  ^ Style/SymbolArray: Use `%i` or `%I` for an array of symbols.
