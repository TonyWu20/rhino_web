# web-2D_model_adsorption_searcher

A web application of 2D model adsorption searcher to be deployed on a web server.

This is a roadmap document of the project.

## Necessary modules

### Backend

Basically, the initial workable version of a backend has been developed.
But it requires a revision to make it handle errors better in the production stage.

- [x] Model Parser
  - [x] `.cell`
  - [x] `.msi`
  - [ ] **`.cif`**
  - [ ] Other
- [x] Adsorption site searcher
- [x] Seed file generator
- [ ] (Optional) Pathway templates for CO2RR

### Frontend

We need a website page to complete the following functions:

- [x] Upload files
  - [ ] Verifications of the uploaded file formats and contents
- [x] Communicate with the backend, including:
  - [x] Parameter inputs
  - [x] Message outputs
- [x] Client-side form validation
  - [x] Input validation
    - [x] Element symbol
- [ ] Download generated seed files
- [ ] Future: Add a basic 3D renderer for the display and interaction of the uploaded model.

### Server

We need a server runtime to handle the following issues:

- [x] Hosts the backend application.
- [ ] Maintains a file system. (Is it necessary to save uploaded file to local first?)
- [x] Allows file uploads and downloads.
- [x] Communicates with frontend
