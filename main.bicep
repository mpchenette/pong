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
    reserved: true
    serverFarmId: asp.id
    siteConfig: {
      alwaysOn: true
      linuxFxVersion: 'DOCKER|index.docker.io/mpchenette/duopong:latest'
    }
  }
}

resource appSettings 'Microsoft.Web/sites/config@2023-01-01' = {
  parent: app
  name: 'appsettings'
  properties: {

    DOCKER_REGISTRY_SERVER_URL: 'https://index.docker.io'
    WEBSITES_PORT: '8000'
  }
}
