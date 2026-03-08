blarg = if true
^^^^^^^^^^^^^^^ Layout/MultilineAssignmentLayout: Right hand side of multi-line assignment is on the same line as the assignment operator `=`.
         'yes'
       else
         'no'
       end

result = if condition
^^^^^^^^^^^^^^^^^^^^^^^^ Layout/MultilineAssignmentLayout: Right hand side of multi-line assignment is on the same line as the assignment operator `=`.
           do_thing
         else
           other_thing
         end

value = case x
^^^^^^^^^^^^^^ Layout/MultilineAssignmentLayout: Right hand side of multi-line assignment is on the same line as the assignment operator `=`.
        when :a
          1
        else
          2
        end

memoized ||= begin
^^^^^^^^^^^^^^^^^^ Layout/MultilineAssignmentLayout: Right hand side of multi-line assignment is on the same line as the assignment operator `=`.
               build_value
             end

result = fetch_records do
^^^^^^^^^^^^^^^^^^^^^^^^^ Layout/MultilineAssignmentLayout: Right hand side of multi-line assignment is on the same line as the assignment operator `=`.
           build_record
         end
