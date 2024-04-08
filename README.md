# web-2D_model_adsorption_searcher

A web application of 2D model adsorption searcher to be deployed on a web server.

This is a roadmap document of the project.

## Necessary modules

### Backend

Basically, the initial workable version of a backend has been developed.

- [x] Model Parser
  - [x] `.cell`
  - [x] `.msi`
  - [ ] Other
- [x] Adsorption site searcher
- [x] Seed file generator
- [ ] (Optional) Pathway templates for CO2RR

### Frontend

We need a website page to complete the following functions:

- [ ] Upload files
  - [ ] Verifications of the uploaded file formats and contents
- [ ] Communicate with the backend, including:
  - [ ] Parameter inputs
  - [ ] Message outputs
- [ ] Download generated seed files

### Server

We need a server runtime to handle the following issues:

- [ ] Hosts the backend application.
- [ ] Maintains a file system.
- [ ] Allows file uploads and downloads.
