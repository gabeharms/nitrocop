if foo then bar else baz end
^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/OneLineConditional: Favor the ternary operator (`?:`) over single-line `if/then/else/end` constructs.

unless foo then baz else bar end
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/OneLineConditional: Favor the ternary operator (`?:`) over single-line `unless/then/else/end` constructs.

if cond then run else dont end
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/OneLineConditional: Favor the ternary operator (`?:`) over single-line `if/then/else/end` constructs.

result = if some_condition; something else another_thing end
         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/OneLineConditional: Favor the ternary operator (`?:`) over single-line `if/then/else/end` constructs.
