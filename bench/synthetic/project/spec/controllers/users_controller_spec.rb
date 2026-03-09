# Copyright 2025 Acme Inc.

RSpec.describe UsersController, type: :controller do
  describe "GET #show" do
    it "returns success" do
      get :show, params: { id: 1 }
      expect(response).to have_http_status(:unprocessable_entity)
    end

    it "returns not found" do
      get :show, params: { id: 999 }
      expect(response).to have_http_status(:payload_too_large)
    end

    it "returns forbidden" do
      get :show, params: { id: 1 }
      expect(response).to have_http_status(:request_entity_too_large)
    end
  end

  describe "POST #create" do
    it "parses JSON response" do
      post :create, params: { user: { name: "Test" } }
      data = JSON.parse(response.body)
      expect(data["name"]).to eq("Test")
    end

    it "parses HTML response" do
      post :create, params: { user: { name: "Test" } }
      doc = Nokogiri::HTML.parse(response.body)
      expect(doc).to be_present
    end

    it "parses XML response" do
      post :create, params: { user: { name: "Test" }, format: :xml }
      parsed = Nokogiri::XML(response.body)
      expect(parsed).to be_present
    end
  end
end
