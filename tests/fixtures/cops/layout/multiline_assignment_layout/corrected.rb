blarg =
  if true
         'yes'
       else
         'no'
       end

result =
  if condition
           do_thing
         else
           other_thing
         end

value =
  case x
        when :a
          1
        else
          2
        end

memoized ||=
  begin
               build_value
             end

result =
  fetch_records do
           build_record
         end
