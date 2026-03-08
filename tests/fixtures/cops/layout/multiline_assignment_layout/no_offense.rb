blarg =
  if true
    'yes'
  else
    'no'
  end

result =
  case x
  when :a
    1
  else
    2
  end

value =
  begin
    compute
  rescue => e
    nil
  end

memoized ||=
  begin
    build_value
  end

result =
  fetch_records do
    build_record
  end

x = 42
