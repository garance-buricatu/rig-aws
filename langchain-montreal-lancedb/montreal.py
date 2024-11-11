import aiohttp
from typing import List

# Constants
MONTREAL_OPEN_DATA_API = "https://donnees.montreal.ca/api/3/action"
PAGE_SIZE = 50

class CategoryMetadata:
    def __init__(self, item: dict):
        self.id = item.get('id', '')
        self.titre = item.get('title', '')
        self.tags = [tag.get('name', '') for tag in item.get('tags', [])]
        self.groupes = [group.get('name', '') for group in item.get('groups')]
        self.organisation = item.get('organization', {}).get('name', '')
        self.notes = item.get('notes', '')
        self.territoire = item.get('territoire', '')
        self.description_donnees = [resource.get('description', '') for resource in item.get('resources', [])]
        self.methodologie = item.get('methodologie', '')

    def to_paragraph(self) -> str:
        # Create a paragraph string
        paragraph = ""
        paragraph += f"{self.titre}. "
        paragraph += f"{self.notes}"
        paragraph += f"{', '.join(self.description_donnees)}"
        paragraph += f"{self.methodologie}"
        paragraph += f"Organisation: {self.organisation}. "
        paragraph += f"Territoire comprend: {', '.join(self.territoire)}. "
        paragraph += f"Groupes comprend: {', '.join(self.groupes)}. "
        paragraph += f"Tags: {', '.join(self.tags)}. "
        return paragraph

# Client to fetch data
class MontrealOpenDataClient:
    def __init__(self, url: str = MONTREAL_OPEN_DATA_API):
        self.url = url

    async def fetch_page(self, session, skip: int) -> List[dict]:
        """Fetches a single page of data."""
        async with session.get(f"{self.url}/package_search", params={"start": skip, "rows": PAGE_SIZE}) as response:
            data = await response.json()
            return data["result"]["results"]
            

    async def search_all(self) -> List[dict]:
        """Fetches all pages of data until no more items are found."""
        async with aiohttp.ClientSession() as session:
            all_items = []
            skip = 0

            while True:
                items = await self.fetch_page(session, skip)
                if not items:
                    break
                all_items.extend(items)
                skip += PAGE_SIZE

            return all_items
