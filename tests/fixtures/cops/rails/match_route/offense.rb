match ':controller/:action/:id'
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/MatchRoute: Use `get` instead of `match` to define a route.
match 'photos/:id', to: 'photos#show', via: :get
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/MatchRoute: Use `get` instead of `match` to define a route.
match 'users/:id', to: 'users#update', via: :patch
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/MatchRoute: Use `patch` instead of `match` to define a route.
match 'photos/:id', to: 'photos#show', via: [:get]
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/MatchRoute: Use `get` instead of `match` to define a route.
match '/audits/auto_complete_search' => 'audits#auto_complete_search', :via => [:get]
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/MatchRoute: Use `get` instead of `match` to define a route.
match '/audits' => 'react#index', :via => [:get]
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/MatchRoute: Use `get` instead of `match` to define a route.
match 'users/:id', to: 'users#update', via: [:patch]
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/MatchRoute: Use `patch` instead of `match` to define a route.
match "login_#{role}/:id" => "sessions#login_#{role}"
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/MatchRoute: Use `get` instead of `match` to define a route.
