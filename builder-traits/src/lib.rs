use either::Either;

pub trait Table {}
pub trait Column {
    type Table: Table;
    type Type;
}

pub trait Buildable {
    type Builder;
}

pub trait BuildableTable: Table {
    type Insertable: Buildable;
}

pub trait DescendantTable: BuildableTable {
    type ParentTables: Buildable;
}

pub trait SetColumn<C: Column> {
    fn set_column(self, value: <C as Column>::Type) -> Either<Self, (Self, <C as Column>::Type)>
    where
        Self: Sized;
}

pub struct GenericBuilder<T: DescendantTable> {
    extended_table_builders: <<T as DescendantTable>::ParentTables as Buildable>::Builder,
    table_builder: <<T as BuildableTable>::Insertable as Buildable>::Builder,
}

impl<C, T> SetColumn<C> for GenericBuilder<T>
where
    C: Column,
    T: DescendantTable,
    <<T as BuildableTable>::Insertable as Buildable>::Builder: SetColumn<C>,
    <<T as DescendantTable>::ParentTables as Buildable>::Builder: SetColumn<C>,
{
    fn set_column(mut self, value: <C as Column>::Type) -> Either<Self, (Self, <C as Column>::Type)> {
        match self.table_builder.set_column(value) {
            Either::Left(tb) => {
                self.table_builder = tb;
                Either::Left(self)
            }
            Either::Right((tb, v)) => {
                self.table_builder = tb;
                match self.extended_table_builders.set_column(v) {
                    Either::Left(etb) => {
                        self.extended_table_builders = etb;
                        Either::Left(self)
                    }
                    Either::Right((etb, v)) => {
                        self.extended_table_builders = etb;
                        Either::Right((self, v))
                    }
                }
            }
        }
    }
}
