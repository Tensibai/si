async function exists(input: Input): Promise<Output> {
  if (input.resource?.value && input.deleted_at) {
    return {
      success: false,
      recommendedActions: ["delete"]
    }
  }
  return {
    success: true,
    recommendedActions: [],
  }
}