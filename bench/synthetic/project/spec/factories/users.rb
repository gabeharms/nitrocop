# Copyright 2025 Acme Inc.

FactoryBot.define do
  # IdSequence
  factory :user do
    sequence :id
    name { "Test User" }
  end

  factory :post do
    sequence(:id, 1000)
    title { "Test Post" }
  end

  factory :comment do
    sequence(:id, (1..10).cycle)
    body { "Test Comment" }
  end
end
