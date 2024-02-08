use async_graphql::{Context, EmptySubscription, FieldResult, Object, Schema};

use crate::{
    config::mongo::DbMongo,
    schemas::project_schema::{
        CreateOwner, CreateProject, FetchOwner, FetchProject, Owner, Project,
    },
};

pub struct Query;

#[Object(extends)]
impl Query {
    async fn owner(&self, ctx: &Context<'_>, input: FetchOwner) -> FieldResult<Owner> {
        let db = &ctx.data_unchecked::<DbMongo>();
        let owner = db.single_owner(&input._id).await.unwrap();
        Ok(owner)
    }

    async fn get_owners(&self, ctx: &Context<'_>) -> FieldResult<Vec<Owner>> {
        let db = &ctx.data_unchecked::<DbMongo>();
        let owner = db.get_owners().await.unwrap();
        Ok(owner)
    }

    //projects query
    async fn project(&self, ctx: &Context<'_>, input: FetchProject) -> FieldResult<Project> {
        let db = &ctx.data_unchecked::<DbMongo>();
        let project = db.single_project(&input._id).await.unwrap();
        Ok(project)
    }

    async fn get_projects(&self, ctx: &Context<'_>) -> FieldResult<Vec<Project>> {
        let db = &ctx.data_unchecked::<DbMongo>();
        let projects = db.get_projects().await.unwrap();
        Ok(projects)
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    //owner mutation
    async fn create_owner(&self, ctx: &Context<'_>, input: CreateOwner) -> FieldResult<Owner> {
        let db = &ctx.data_unchecked::<DbMongo>();
        let new_owner = Owner {
            _id: None,
            email: input.email,
            name: input.name,
            phone: input.phone,
        };
        let owner = db.create_owner(new_owner).await.unwrap();
        Ok(owner)
    }

    async fn create_project(
        &self,
        ctx: &Context<'_>,
        input: CreateProject,
    ) -> FieldResult<Project> {
        let db = &ctx.data_unchecked::<DbMongo>();
        let new_project = Project {
            _id: None,
            owner_id: input.owner_id,
            name: input.name,
            description: input.description,
            status: input.status,
        };
        let project = db.create_project(new_project).await.unwrap();
        Ok(project)
    }
}

pub type GraphqlSchema = Schema<Query, Mutation, EmptySubscription>;
