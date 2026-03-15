# Copyright 2025 Acme Inc.

Rails.application.routes.draw do
  # MultipleRoutePaths
  get '/users', '/people', to: 'users#index'
  post '/items', '/products', to: 'items#create'
  put '/a', '/b', '/c', to: 'misc#update'
end
