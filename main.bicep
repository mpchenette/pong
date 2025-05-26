param location string = resourceGroup().location

resource asp 'Microsoft.Web/serverfarms@2023-01-01' = {
  name: 'asp-mpchenette-prod-southcentralus-001'
  location: location
  sku: {
    capacity: 1
    family: 'B'
    name: 'B1'
    size: 'B1'
    tier: 'Basic'
  }
  properties: {
    reserved: true
  }
}

resource app 'Microsoft.Web/sites@2023-01-01' = {
  name: 'mpchenette'
  location: location
  kind: 'app,linux,container'
  properties: {
    // httpsOnly: false
    reserved: true
    serverFarmId: asp.id
    siteConfig: {
      // acrUseManagedIdentityCreds: true
      alwaysOn: true
      linuxFxVersion: 'DOCKER|index.docker.io/mpchenette/rust-std-web-server:latest'
      // appSettings: [
      //   {
      //     name: 'DOCKER_REGISTRY_SERVER_URL'
      //     value: 'https://index.docker.io'
      //   }
      // ]
    }
  }
}

resource appSettings 'Microsoft.Web/sites/config@2023-01-01' = {
  parent: app
  name: 'appsettings'
  properties: {

    DOCKER_REGISTRY_SERVER_URL: 'https://index.docker.io'
    WEBSITES_PORT: '8000'
    // DOCKER_REGISTRY_SERVER_USERNAME: cr.listCredentials().username
    // DOCKER_REGISTRY_SERVER_PASSWORD: cr.listCredentials().passwords[0].value
  }
}