# nitrocop-expect: 5:8 Metrics/BlockNesting: Avoid more than 3 levels of block nesting.
# nitrocop-expect: 6:10 Metrics/BlockNesting: Avoid more than 3 levels of block nesting.
def foo
  if a
    if b
      while c
        if d # rubocop:disable Metrics/BlockNesting
          if e
            x
          end
        end
      end
    end
  end
end
