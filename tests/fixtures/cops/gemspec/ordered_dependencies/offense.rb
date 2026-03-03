# nitrocop-filename: example.gemspec
Gem::Specification.new do |spec|
  spec.add_dependency 'foo', '~> 1.0'
  spec.add_dependency 'bar', '~> 2.0'
       ^^^^^^^^^^^^^^ Gemspec/OrderedDependencies: Dependencies should be sorted in an alphabetical order within their section of the gemspec. Dependency `bar` should appear before `foo`.
  spec.add_dependency 'aaa'
       ^^^^^^^^^^^^^^ Gemspec/OrderedDependencies: Dependencies should be sorted in an alphabetical order within their section of the gemspec. Dependency `aaa` should appear before `bar`.

  spec.add_development_dependency 'zebra'
  spec.add_development_dependency 'alpha'
       ^^^^^^^^^^^^^^^^^^^^^^^^^^ Gemspec/OrderedDependencies: Dependencies should be sorted in an alphabetical order within their section of the gemspec. Dependency `alpha` should appear before `zebra`.

  s.add_runtime_dependency(%q<tilt>, ["~> 1.4"])
  s.add_runtime_dependency(%q<activesupport>, ["~> 4.2"])
     ^^^^^^^^^^^^^^^^^^^^^^ Gemspec/OrderedDependencies: Dependencies should be sorted in an alphabetical order within their section of the gemspec. Dependency `activesupport` should appear before `tilt`.

  s.add_dependency(%Q<zebra>)
  s.add_dependency(%Q<alpha>)
     ^^^^^^^^^^^^^^ Gemspec/OrderedDependencies: Dependencies should be sorted in an alphabetical order within their section of the gemspec. Dependency `alpha` should appear before `zebra`.

  s.add_dependency(%q(zoo))
  s.add_dependency(%q(aaa))
     ^^^^^^^^^^^^^^ Gemspec/OrderedDependencies: Dependencies should be sorted in an alphabetical order within their section of the gemspec. Dependency `aaa` should appear before `zoo`.
end
