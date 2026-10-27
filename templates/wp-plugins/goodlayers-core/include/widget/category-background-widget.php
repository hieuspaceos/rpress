<?php
	/**
	 * A widget that show list of categories.
	 */

	add_action('widgets_init', 'gdlr_core_category_background_widget');
	if( !function_exists('gdlr_core_category_background_widget') ){
		function gdlr_core_category_background_widget() {
			register_widget( 'Goodlayers_Core_Category_Background_Widget' );
		}
	}

	if( !class_exists('Goodlayers_Core_Category_Background_Widget') ){
		class Goodlayers_Core_Category_Background_Widget extends WP_Widget{

			// Initialize the widget
			function __construct() {

				parent::__construct(
					'gdlr-core-category-background-widget', 
					esc_html__('Category Background Widget ( Goodlayers )', 'goodlayers-core'), 
					array('description' => esc_html__('A widget that category list', 'goodlayers-core'))
				);  

			}

			// Output of the widget
			function widget( $args, $instance ) {
	
				$title = empty($instance['title'])? '': apply_filters('widget_title', $instance['title']);
				$categories = empty($instance['category'])? array(): explode(',', $instance['category']);
					
				// Opening of widget
				echo gdlr_core_escape_content($args['before_widget']);
				
				// Open of title tag
				if( !empty($title) ){ 
					echo gdlr_core_escape_content($args['before_title'] . $title . $args['after_title']); 
				}
					
				echo '<ul class="gdlr-core-category-background-widget" >';
				foreach( $categories as $cat_name ){
					$category = get_term_by('slug', $cat_name, 'category');
					$thumbnail = get_term_meta($category->term_id, 'thumbnail', true);

					if( empty($thumbnail) ){
						echo '<li class="gdlr-core-no-bg" >';
					}else{
						echo '<li class="gdlr-core-with-bg" ' . gdlr_core_esc_style(array(
							'background-image' => $thumbnail
						)) . ' >';
					}	
					echo '<a href="' . esc_url(get_term_link($category->term_id, 'category')) . '" >';
					echo '<span class="gdlr-core-category-background-widget-content" >';
					echo '<span class="gdlr-core-category-background-widget-title" >' . $category->name . '</span>';
					echo '<span class="gdlr-core-category-background-widget-count" >' . $category->count . '</span>';
					echo '</span>';
					echo '</a>';
					echo '</li>';
				}
				echo '</ul>';
				
				// Closing of widget
				echo gdlr_core_escape_content($args['after_widget']);

			}

			// Widget Form
			function form( $instance ) {

				$menus = wp_get_nav_menus();
				$menu_options = array();
				foreach( $menus as $menu ){
					$menu_options[$menu->term_id] = $menu->name;
				}

				if( class_exists('gdlr_core_widget_util') ){
					gdlr_core_widget_util::get_option(array(
						'title' => array(
							'type' => 'text',
							'id' => $this->get_field_id('title'),
							'name' => $this->get_field_name('title'),
							'title' => esc_html__('Title', 'goodlayers-core'),
							'value' => (isset($instance['title'])? $instance['title']: '')
						),
						'category' => array(
							'type' => 'multi-combobox',
							'id' => $this->get_field_id('category'),
							'name' => $this->get_field_name('category'),
							'title' => esc_html__('Category', 'goodlayers-core'),
							'options' => gdlr_core_get_term_list('category', ''),
							'value' => (isset($instance['category'])? $instance['category']: '')
						),
					));
				}

			}
			
			// Update the widget
			function update( $new_instance, $old_instance ) {

				if( class_exists('gdlr_core_widget_util') ){
					return gdlr_core_widget_util::get_option_update($new_instance);
				}

				return $new_instance;
			}	
		} // class
	} // class_exists
?>