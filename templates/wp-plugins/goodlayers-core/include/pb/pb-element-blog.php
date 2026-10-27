<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('blog', 'gdlr_core_pb_element_blog'); 
	
	if( !class_exists('gdlr_core_pb_element_blog') ){
		class gdlr_core_pb_element_blog{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-newspaper-o',
					'title' => esc_html__('Blog', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return apply_filters('gdlr_core_blog_item_options', array(					
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'category' => array(
								'title' => esc_html__('Category', 'goodlayers-core'),
								'type' => 'multi-combobox',
								'options' => gdlr_core_get_term_list('category'),
								'description' => esc_html__('You can use Ctrl/Command button to select multiple items or remove the selected item. Leave this field blank to select all items in the list.', 'goodlayers-core'),
							),
							'tag' => array(
								'title' => esc_html__('Tag', 'goodlayers-core'),
								'type' => 'multi-combobox',
								'options' => gdlr_core_get_term_list('post_tag')
							),
							'relation' => array(
								'title' => esc_html__('Relation (Category & Tag)', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'or' => esc_html__('OR', 'goodlayers-core'),
									'and' => esc_html__('AND', 'goodlayers-core')
								)
							),
							'num-fetch' => array(
								'title' => esc_html__('Num Fetch', 'goodlayers-core'),
								'type' => 'text',
								'default' => 9,
								'data-input-type' => 'number',
								'description' => esc_html__('The number of posts showing on the blog item', 'goodlayers-core')
							),
							'prepend-sticky' => array(
								'title' => esc_html__('Prepend Sticky Post', 'goodlayers-core'),
								'type' => 'checkbox',
								'description' => esc_html__('Prepend sticky post at the top of the post.', 'goodlayers-core')
							),
							'orderby' => array(
								'title' => esc_html__('Order By', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'date' => esc_html__('Publish Date', 'goodlayers-core'), 
									'title' => esc_html__('Title', 'goodlayers-core'), 
									'rand' => esc_html__('Random', 'goodlayers-core'), 
									'menu_order' => esc_html__('Menu Order', 'goodlayers-core'), 
								)
							),
							'order' => array(
								'title' => esc_html__('Order', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'desc'=>esc_html__('Descending Order', 'goodlayers-core'), 
									'asc'=> esc_html__('Ascending Order', 'goodlayers-core'), 
								)
							),
							'filterer' => array(
								'title' => esc_html__('Category Filterer', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'none'=>esc_html__('None', 'goodlayers-core'), 
									'text'=>esc_html__('Filter Text Style', 'goodlayers-core'), 
									'button'=>esc_html__('Filter Button Style', 'goodlayers-core'), 
								),
								'description' => esc_html__('Filter is not supported and will be automatically disabled on "blog feature style" and "blog carousel layout".', 'goodlayers-core'),
							),
							'filterer-align' => array(
								'title' => esc_html__('Filterer Alignment', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'text-align',
								'default' => 'center',
								'condition' => array('filterer' => array('text', 'button'))
							),
							'pagination' => array(
								'title' => esc_html__('Pagination', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'none'=>esc_html__('None', 'goodlayers-core'), 
									'page'=>esc_html__('Page', 'goodlayers-core'), 
									'load-more'=>esc_html__('Load More', 'goodlayers-core'), 
								),
								'description' => esc_html__('Pagination is not supported and will be automatically disabled on "blog feature style" and "blog carousel layout".', 'goodlayers-core'),
							),
							'pagination-style' => array(
								'title' => esc_html__('Pagination Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'default' => esc_html__('Default', 'goodlayers-core'),
									'plain' => esc_html__('Plain', 'goodlayers-core'),
									'rectangle' => esc_html__('Rectangle', 'goodlayers-core'),
									'rectangle-border' => esc_html__('Rectangle Border', 'goodlayers-core'),
									'round' => esc_html__('Round', 'goodlayers-core'),
									'round-border' => esc_html__('Round Border', 'goodlayers-core'),
									'circle' => esc_html__('Circle', 'goodlayers-core'),
									'circle-border' => esc_html__('Circle Border', 'goodlayers-core'),
								),
								'default' => 'default',
								'condition' => array( 'pagination' => 'page' )
							),
							'pagination-align' => array(
								'title' => esc_html__('Pagination Alignment', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'text-align',
								'with-default' => true,
								'default' => 'default',
								'condition' => array( 'pagination' => 'page' )
							),
							'offset' => array(
								'title' => esc_html__('Query Offset', 'goodlayers-core'),
								'type' => 'text',
								'default' => 0,
								'data-input-type' => 'number',
								'condition' => array( 'pagination' => 'none' ),
								'description' => esc_html__('The number of posts you want to skip, cannot be used with pagination.', 'goodlayers-core')
							)	
						),
					),
					'settings' => array(
						'title' => esc_html__('Blog Style', 'goodlayers-core'),
						'options' => array(
							'blog-style' => array(
								'title' => esc_html__('Blog Style', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => array(
									'blog-full' => GDLR_CORE_URL . '/include/images/blog-style/blog-full.png',
									'blog-full-with-frame' => GDLR_CORE_URL . '/include/images/blog-style/blog-full-with-frame.png',
									'blog-column' => GDLR_CORE_URL . '/include/images/blog-style/blog-column.png',
									'blog-column-with-frame' => GDLR_CORE_URL . '/include/images/blog-style/blog-column-with-frame.png',
									'blog-column-no-space' => GDLR_CORE_URL . '/include/images/blog-style/blog-column-no-space.png',
									'blog-column-hover-bg' => GDLR_CORE_URL .'/include/images/blog-style/blog-column-hover-bg.jpg',
									'blog-image' => GDLR_CORE_URL . '/include/images/blog-style/blog-image.png',
									'blog-image-no-space' => GDLR_CORE_URL . '/include/images/blog-style/blog-image-no-space.png',
									'blog-left-thumbnail' => GDLR_CORE_URL . '/include/images/blog-style/blog-left-thumbnail.png',
									'blog-right-thumbnail' => GDLR_CORE_URL . '/include/images/blog-style/blog-right-thumbnail.png',
									'blog-metro' => GDLR_CORE_URL . '/include/images/blog-style/blog-metro.png',
									'blog-metro-no-space' => GDLR_CORE_URL . '/include/images/blog-style/blog-metro-no-space.png',
									'blog-list' => GDLR_CORE_URL . '/include/images/blog-style/blog-list.png',
									'blog-list-center' => GDLR_CORE_URL . '/include/images/blog-style/blog-list-center.png',
									'blog-widget' => GDLR_CORE_URL . '/include/images/blog-style/blog-widget.png',
									'blog-feature' => GDLR_CORE_URL . '/include/images/blog-style/blog-feature.png',
								),
								'default' => 'blog-full',
								'wrapper-class' => 'gdlr-core-fullsize'
							),
							'blog-full-style' => array(
								'title' => esc_html__('Blog Full Style', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => array(
									'style-1' => GDLR_CORE_URL . '/include/images/blog-style/blog-full/style-1.jpg',
									'style-2' => GDLR_CORE_URL . '/include/images/blog-style/blog-full/style-2.jpg',
									'style-2-date' => GDLR_CORE_URL . '/include/images/blog-style/blog-full/style-3.jpg',
									'style-2-tag' => GDLR_CORE_URL . '/include/images/blog-style/blog-full/style-5.jpg',
									'style-3' => GDLR_CORE_URL . '/include/images/blog-style/blog-full/style-4.jpg',
								),
								'condition' => array( 'blog-style'=>array('blog-full', 'blog-full-with-frame') ),
								'wrapper-class' => 'gdlr-core-fullsize'
							),
							'blog-side-thumbnail-style' => array(
								'title' => esc_html__('Blog Side Thumbnail Style', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => array(
									'style-1' => GDLR_CORE_URL . '/include/images/blog-style/blog-medium/style-1.jpg',
									'style-1-large' => GDLR_CORE_URL . '/include/images/blog-style/blog-medium/style-1-large.jpg',
									'style-2' => GDLR_CORE_URL . '/include/images/blog-style/blog-medium/style-2.jpg',
									'style-2-large' =>  GDLR_CORE_URL . '/include/images/blog-style/blog-medium/style-2-large.jpg',
								),
								'condition' => array( 'blog-style'=>array('blog-left-thumbnail', 'blog-right-thumbnail') ),
								'wrapper-class' => 'gdlr-core-fullsize'
							),
							'blog-column-style' => array(
								'title' => esc_html__('Blog Column Style', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => array(
									'style-1' => GDLR_CORE_URL . '/include/images/blog-style/blog-column/style-1.jpg',
									'style-2' => GDLR_CORE_URL . '/include/images/blog-style/blog-column/style-2.jpg',
									'style-2-date' => GDLR_CORE_URL . '/include/images/blog-style/blog-column/style-2-date.jpg',
									'style-2-tag' => GDLR_CORE_URL . '/include/images/blog-style/blog-column/style-2-tag.jpg',
									'style-3' => GDLR_CORE_URL . '/include/images/blog-style/blog-column/style-3.jpg',
									'style-3-date' => GDLR_CORE_URL . '/include/images/blog-style/blog-column/style-3-date.jpg',
									'style-4' => GDLR_CORE_URL . '/include/images/blog-style/blog-column/style-4.jpg',
									'style-4-left-button' => GDLR_CORE_URL . '/include/images/blog-style/blog-column/style-4-left-button.jpg',
								),
								'condition' => array( 'blog-style'=>array('blog-column', 'blog-column-with-frame', 'blog-column-no-space') ),
								'wrapper-class' => 'gdlr-core-fullsize'
							),
							'blog-column-text-align' => array(
								'title' => esc_html__('Blog Column Text Align', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'text-align',
								'default' => 'left',
								'condition' => array( 'blog-style' => array('blog-column', 'blog-column-with-frame', 'blog-column-no-space') )
							),
							'blog-image-style' => array(
								'title' => esc_html__('Blog Image Style', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => array(
									'style-1' => GDLR_CORE_URL . '/include/images/blog-style/blog-modern/style-1.jpg',
									'style-2' => GDLR_CORE_URL . '/include/images/blog-style/blog-modern/style-2.jpg',
									'style-3' => GDLR_CORE_URL . '/include/images/blog-style/blog-modern/style-3.jpg',
									'style-4' => GDLR_CORE_URL . '/include/images/blog-style/blog-modern/style-4.jpg',
									'style-5' => GDLR_CORE_URL . '/include/images/blog-style/blog-modern/style-5.jpg',
								),
								'condition' => array( 'blog-style'=>array('blog-image', 'blog-image-no-space') ),
								'wrapper-class' => 'gdlr-core-fullsize'
							),
							'blog-image-5-background' => array(
								'title' => esc_html__('Blog Image 5 Background', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'blog-style'=>array('blog-image', 'blog-image-no-space'), 'blog-image-style' => 'style-5' ),
							),
							'blog-widget-style' => array(
								'title' => esc_html__('Blog Widget Style', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => array(
									'style-1' => GDLR_CORE_URL . '/include/images/blog-style/blog-widget/style-1.jpg',
									'style-2' => GDLR_CORE_URL . '/include/images/blog-style/blog-widget/style-2.jpg',
									'style-3' => GDLR_CORE_URL . '/include/images/blog-style/blog-widget/style-3.jpg',
								),
								'condition' => array( 'blog-style' => 'blog-widget' ),
								'wrapper-class' => 'gdlr-core-fullsize'
							),
							'blog-widget-with-feature' => array(
								'title' => esc_html__('Blog Widget With Feature', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable',
								'condition' => array( 'blog-style' => 'blog-widget' )
							),
							'blog-full-alignment' => array(
								'title' => esc_html__('Blog Full Alignment', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'left' => esc_html__('Left', 'goodlayers-core'),
									'center' => esc_html__('Center', 'goodlayers-core'),
								),
								'condition' => array( 'blog-style' => array('blog-full', 'blog-full-with-frame') )
							),
							'blog-image-alignment' => array(
								'title' => esc_html__('Blog Image Alignment', 'goodlayers-core'),
								'type' => 'combobox',
								'default' => 'center',
								'options' => array(
									'left' => esc_html__('Left', 'goodlayers-core'),
									'center' => esc_html__('Center', 'goodlayers-core'),
								),
								'condition' => array( 'blog-style' => array('blog-image', 'blog-image-no-space'), 'blog-image-style' => array('style-1', 'style-2') )
							),
							'blog-list-with-frame' => array(
								'title' => esc_html__('Blog List With Frame', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable',
								'condition' => array( 'blog-style' => array('blog-list', 'blog-list-center') )
							),
							'always-show-overlay-content' => array(
								'title' => esc_html__('Always Show Overlay Content', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable',
								'condition' => array( 'blog-style' => array('blog-image', 'blog-image-no-space', 'blog-feature') )
							),
							'blog-image-thumbnail-overlay' => array(
								'title' => esc_html__('Blog Image Thumbnail Overlay', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'none' => esc_html__('None', 'goodlayers-core'),
									'gradient' => esc_html__('Bottom Gradient', 'goodlayers-core'),
									'gradient2' => esc_html__('Bottom Gradient 2', 'goodlayers-core'),
									'gradient-slide' => esc_html__('Bottom Gradient (Slide Up)', 'goodlayers-core'),
									'opacity' => esc_html__('Opacity', 'goodlayers-core')
								),
								'default' => 'enable',
								'condition' => array( 'blog-style' => array('blog-image', 'blog-image-no-space', 'blog-feature') )
							),
							'blog-image-thumbnail-overlay-opacity' => array(
								'title' => esc_html__('Blog Image Thumbnail Overlay Opacity', 'goodlayers-core'),
								'type' => 'text',
								'default' => '0.4',
								'condition' => array( 'blog-style' => array('blog-image', 'blog-image-no-space'), 'blog-image-thumbnail-overlay' => 'opacity' ),
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core')
							),
							'blog-image-thumbnail-overlay-color' => array(
								'title' => esc_html__('Blog Image Thumbnail Overlay Opacity', 'goodlayers-core'),
								'type' => 'colorpicker',
								'default' => '#000000',
								'condition' => array( 'blog-style' => array('blog-image', 'blog-image-no-space'), 'blog-image-thumbnail-overlay' => 'opacity' ),
							),
							'blog-column-thumbnail-overlay-opacity' => array(
								'title' => esc_html__('Blog Column Background Thumbnail Overlay Opacity', 'goodlayers-core'),
								'type' => 'text',
								'default' => '0.4',
								'condition' => array( 'blog-style' => array('blog-column-hover-bg') ),
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core')
							),
							'blog-column-thumbnail-overlay-color' => array(
								'title' => esc_html__('Blog Column Background Thumbnail Overlay Opacity', 'goodlayers-core'),
								'type' => 'colorpicker',
								'default' => '#000000',
								'condition' => array( 'blog-style' => array('blog-column-hover-bg') ),
							),
							'show-thumbnail' => array(
								'title' => esc_html__('Show Thumbnail', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'enable',
								'condition' => array( 'blog-style' => array('blog-full', 'blog-full-with-frame', 'blog-column', 'blog-column-with-frame', 'blog-column-no-space', 'blog-left-thumbnail', 'blog-right-thumbnail', 'blog-widget') )
							),
							'thumbnail-size' => array(
								'title' => esc_html__('Thumbnail Size', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => 'thumbnail-size',
								'condition' => array( 'blog-style' => array('blog-full', 'blog-full-with-frame', 'blog-column', 'blog-column-with-frame', 'blog-column-no-space', 'blog-image', 'blog-image-no-space', 'blog-left-thumbnail', 'blog-right-thumbnail', 'blog-metro', 'blog-metro-no-space', 'blog-widget', 'blog-feature') , 'show-thumbnail' => array( 'enable' ) )
							),
							'enable-thumbnail-opacity-on-hover' => array(
								'title' => esc_html__('Thumbnail Opacity on Hover', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'enable',
								'condition' => array( 'blog-style' => array('blog-full', 'blog-full-with-frame', 'blog-column', 'blog-column-with-frame', 'blog-column-no-space', 'blog-image', 'blog-image-no-space', 'blog-left-thumbnail', 'blog-right-thumbnail', 'blog-metro', 'blog-metro-no-space', 'blog-widget', 'blog-feature') , 'show-thumbnail' => array( 'enable' ) )
							),
							'blog-image-initial-opacity' => array(
								'title' => esc_html__('Blog Image Initial Opacity', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Fill the number between 0 and 1. Only took effects when "Thumbnail Opacity on Hover" option is enabled.', 'frmaster'),
								'condition' => array('blog-style' => array('blog-image', 'blog-image-no-space'))
							),
							'enable-thumbnail-zoom-on-hover' => array(
								'title' => esc_html__('Thumbnail Zoom on Hover', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'enable' => esc_html__('Zoom', 'goodlayers-core'),
									'zoom-rotate' => esc_html__('Zoom & Rotate', 'goodlayers-core'),
									'disable' => esc_html__('Disable', 'goodlayers-core'),
								),
								'default' => 'enable',
								'condition' => array( 'blog-style' => array('blog-full', 'blog-full-with-frame', 'blog-column', 'blog-column-with-frame', 'blog-column-no-space', 'blog-image', 'blog-image-no-space', 'blog-left-thumbnail', 'blog-right-thumbnail', 'blog-metro', 'blog-metro-no-space', 'blog-widget', 'blog-feature') , 'show-thumbnail' => array( 'enable' ) )
							),
							'enable-thumbnail-grayscale-effect' => array(
								'title' => esc_html__('Enable Thumbnail Grayscale Effect', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable',
								'condition' => array( 'blog-style' => array('blog-full', 'blog-full-with-frame', 'blog-column', 'blog-column-with-frame', 'blog-column-no-space', 'blog-image', 'blog-image-no-space', 'blog-left-thumbnail', 'blog-right-thumbnail', 'blog-metro', 'blog-metro-no-space', 'blog-widget', 'blog-feature') , 'show-thumbnail' => array( 'enable' ) ),
								'description' => esc_html__('Only works with browser that supports css3 filter ( http://caniuse.com/#feat=css-filters ).', 'goodlayers-core')
							),
							'column-size' => array(
								'title' => esc_html__('Column Size', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array( 60 => 1, 30 => 2, 20 => 3, 15 => 4, 12 => 5 ),
								'default' => 20,
								'condition' => array( 'blog-style' => array('blog-column', 'blog-column-with-frame', 'blog-column-no-space', 'blog-column-hover-bg', 'blog-image', 'blog-image-no-space', 'blog-metro', 'blog-metro-no-space') )
							),
							'layout' => array(
								'title' => esc_html__('Layout', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array( 
									'fitrows' => esc_html__('Fit Rows', 'goodlayers-core'),
									'carousel' => esc_html__('Carousel', 'goodlayers-core'),
									'masonry' => esc_html__('Masonry', 'goodlayers-core'),
								),
								'default' => 'fitrows',
								'condition' => array( 'blog-style' => array('blog-full', 'blog-full-with-frame', 'blog-left-thumbnail', 'blog-right-thumbnail', 'blog-column', 'blog-column-with-frame', 'blog-column-no-space', 'blog-column-hover-bg', 'blog-image', 'blog-image-no-space') )
							),
							'carousel-item-margin' => array(
								'title' => esc_html__('Carousel Item Margin', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'condition' => array( 'layout' => 'carousel' )
							),
							'carousel-overflow' => array(
								'title' => esc_html__('Carousel Overflow', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'' => esc_html__('Hidden', 'goodlayers-core'),
									'visible' => esc_html__('Visible', 'goodlayers-core')
								),
								'condition' => array( 'blog-style' => array('blog-full', 'blog-full-with-frame', 'blog-left-thumbnail', 'blog-right-thumbnail', 'blog-column', 'blog-column-with-frame', 'blog-column-no-space', 'blog-image', 'blog-image-no-space', 'blog-column-hover-bg'), 'layout' => 'carousel' )
							),
							'carousel-scrolling-item-amount' => array(
								'title' => esc_html__('Carousel Scrolling Item Amount', 'goodlayers-core'),
								'type' => 'text',
								'default' => '1',
								'condition' => array( 'blog-style' => array('blog-full', 'blog-full-with-frame', 'blog-left-thumbnail', 'blog-right-thumbnail', 'blog-column', 'blog-column-with-frame', 'blog-column-no-space', 'blog-image', 'blog-image-no-space', 'blog-column-hover-bg'), 'layout' => 'carousel' )
							),
							'carousel-autoslide' => array(
								'title' => esc_html__('Autoslide Carousel', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'enable',
								'condition' => array( 'blog-style' => array('blog-full', 'blog-full-with-frame', 'blog-left-thumbnail', 'blog-right-thumbnail', 'blog-column', 'blog-column-with-frame', 'blog-column-no-space', 'blog-image', 'blog-image-no-space', 'blog-column-hover-bg'), 'layout' => 'carousel' )
							),
							'carousel-navigation' => array(
								'title' => esc_html__('Carousel Navigation', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => (function_exists('gdlr_core_get_flexslider_navigation_types')? gdlr_core_get_flexslider_navigation_types(): array()),
								'default' => 'navigation',
								'condition' => array( 'blog-style' => array('blog-full', 'blog-full-with-frame', 'blog-left-thumbnail', 'blog-right-thumbnail', 'blog-column', 'blog-column-with-frame', 'blog-column-no-space', 'blog-image', 'blog-image-no-space', 'blog-column-hover-bg'), 'layout' => 'carousel' )
							),
							'carousel-navigation-show-on-hover' => array(
								'title' => esc_html__('Carousel Navigation Display On Hover', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable',
								'condition' => array( 'carousel-navigation' => array('navigation-outer', 'navigation-inner', 'navigation-inner-bullet') )
							),
							'carousel-navigation-align' => (function_exists('gdlr_core_get_flexslider_navigation_align')? gdlr_core_get_flexslider_navigation_align(): array()),
							'carousel-navigation-left-icon' => (function_exists('gdlr_core_get_flexslider_navigation_left_icon')? gdlr_core_get_flexslider_navigation_left_icon(): array()),
							'carousel-navigation-right-icon' => (function_exists('gdlr_core_get_flexslider_navigation_right_icon')? gdlr_core_get_flexslider_navigation_right_icon(): array()),
							'carousel-navigation-icon-color' => (function_exists('gdlr_core_get_flexslider_navigation_icon_color')? gdlr_core_get_flexslider_navigation_icon_color(): array()),
							'carousel-navigation-icon-hover-color' => (function_exists('gdlr_core_get_flexslider_navigation_icon_hover_color')? gdlr_core_get_flexslider_navigation_icon_hover_color(): array()),
							'carousel-navigation-icon-bg' => (function_exists('gdlr_core_get_flexslider_navigation_icon_background')? gdlr_core_get_flexslider_navigation_icon_background(): array()),
							'carousel-navigation-icon-padding' => (function_exists('gdlr_core_get_flexslider_navigation_icon_padding')? gdlr_core_get_flexslider_navigation_icon_padding(): array()),
							'carousel-navigation-icon-radius' => (function_exists('gdlr_core_get_flexslider_navigation_icon_radius')? gdlr_core_get_flexslider_navigation_icon_radius(): array()),
							'carousel-navigation-size' => (function_exists('gdlr_core_get_flexslider_navigation_icon_size')? gdlr_core_get_flexslider_navigation_icon_size(): array()),
							'carousel-navigation-margin' => (function_exists('gdlr_core_get_flexslider_navigation_margin')? gdlr_core_get_flexslider_navigation_margin(): array()),
							'carousel-navigation-side-margin' => (function_exists('gdlr_core_get_flexslider_navigation_side_margin')? gdlr_core_get_flexslider_navigation_side_margin(): array()),
							'carousel-navigation-icon-margin' => (function_exists('gdlr_core_get_flexslider_navigation_icon_margin')? gdlr_core_get_flexslider_navigation_icon_margin(): array()),
							'carousel-bullet-style' => array(
								'title' => esc_html__('Carousel Bullet Style', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => (function_exists('gdlr_core_get_flexslider_bullet_itypes')? gdlr_core_get_flexslider_bullet_itypes(): array()),
								'condition' => array( 'layout' => 'carousel', 'carousel-navigation' => array('bullet','both','touch-navigation-bullet') ),
								'wrapper-class' => 'gdlr-core-fullsize'
							),
							'carousel-bullet-top-margin' => array(
								'title' => esc_html__('Carousel Bullet Top Margin', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'condition' => array( 'layout' => 'carousel', 'carousel-navigation' => array('bullet','both','touch-navigation-bullet') )
							),
							'blog-widget-column' => array(
								'title' => esc_html__('Blog Widget Column', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array( 60 => 1, 30 => 2, 20 => 3 ),
								'default' => 60,
								'condition' => array( 'blog-style' => 'blog-widget' )
							),
							'blog-widget-bottom-divider' => array(
								'title' => esc_html__('Blog Widget Bottom Divider', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable',
								'condition' => array( 'blog-style' => 'blog-widget' )
							),
							'item-size' => array(
								'title' => esc_html__('Item Size', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array( 
									'small' => esc_html__('Small', 'goodlayers-core'),
									'large' => esc_html__('Large', 'goodlayers-core'),
								),
								'default' => 'small',
								'condition' => array( 'blog-style' => 'blog-widget' )
							),
							'excerpt' => array(
								'title' => esc_html__('Excerpt Type', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'specify-number' => esc_html__('Specify Number', 'goodlayers-core'),
									'show-all' => esc_html__('Show All ( use <!--more--> tag to cut the content )', 'goodlayers-core'),
								),
								'default' => 'specify-number',
								'condition' => array( 'blog-style' => array('blog-full', 'blog-full-with-frame', 'blog-column', 'blog-column-with-frame', 'blog-column-no-space','blog-left-thumbnail', 'blog-right-thumbnail') )
							),
							'excerpt-number' => array(
								'title' => esc_html__('Excerpt Number', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'number',
								'default' => 15,
								'condition' => array( 'blog-style' => array('blog-full', 'blog-full-with-frame', 'blog-column', 'blog-column-with-frame', 'blog-column-no-space', 'blog-left-thumbnail', 'blog-right-thumbnail'), 'excerpt' => 'specify-number' )
							),
							'blog-image-excerpt-number' => array(
								'title' => esc_html__('Blog Image Excerpt Number', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'number',
								'default' => 0,
								'condition' => array( 'blog-style' => array('blog-image', 'blog-image-no-space') )
							),
							'blog-date-feature' => array(
								'title' => esc_html__('Enable Blog Date Feature', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'enable',
								'condition' => array( 'blog-style' => array('blog-full', 'blog-full-with-frame', 'blog-left-thumbnail', 'blog-right-thumbnail') ),
								'description' => esc_html__('This will be ignored on "Blog Side Thumbnail - Style 2", "Blog Full - Style 2", "Blog Full - Style 3"', 'goodlayers-core')
							),
							'meta-option' => array(
								'title' => esc_html__('Meta Option', 'goodlayers-core'),
								'type' => 'multi-combobox',
								'options' => array( 
									'date' => esc_html__('Date', 'goodlayers-core'),
									'author' => esc_html__('Author', 'goodlayers-core'),
									'category' => esc_html__('Category', 'goodlayers-core'),
									'tag' => esc_html__('Tag', 'goodlayers-core'),
									'comment' => esc_html__('Comment', 'goodlayers-core'),
									'comment-number' => esc_html__('Comment Number', 'goodlayers-core'),
								),
								// 'default' => array('date', 'author', 'category')
							),
							'show-read-more' => array(
								'title' => esc_html__('Read More Button', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'disable' => esc_html__('None', 'goodlayers-core'),
									'enable' => esc_html__('Default', 'goodlayers-core'),
									'button' => esc_html__('Button', 'goodlayers-core'),
									'border-button' => esc_html__('Border Button', 'goodlayers-core'),
									'text' => esc_html__('Plain Text', 'goodlayers-core'),
									'text-hover-border' => esc_html__('Plain Text With Hover Border', 'goodlayers-core')
								),
								'default' => 'enable',
								'condition' => array( 'blog-style' => array('blog-full', 'blog-full-with-frame', 'blog-left-thumbnail', 'blog-right-thumbnail', 'blog-column', 'blog-column-no-space', 'blog-column-with-frame'), )
							),
							'read-more-button-text' => array(
								'title' => esc_html__('Read More Button Text', 'goodlayers-core'),
								'type' => 'text',
								'condition' => array( 'show-read-more' => array('enable', 'button', 'text'), 'blog-style' => array('blog-full', 'blog-full-with-frame', 'blog-left-thumbnail', 'blog-right-thumbnail', 'blog-column', 'blog-column-no-space', 'blog-column-with-frame'), )
							)
						),
					),	
					'typography' => array(
						'title' => esc_html__('Typography', 'goodlayers-core'),
						'options' => array(
							'filterer-bottom-margin' => array(
								'title' => esc_html__('Filter Bottom Margin ( If Any )', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'description' => esc_html__('Leave this field blank for default value', 'goodlayers-core')
							),
							'blog-title-font-size' => array(
								'title' => esc_html__('Blog Title Font Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'blog-title-font-style' => array(
								'title' => esc_html__('Blog Title Font Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'normal' => esc_html__('Normal', 'goodlayers-core'),
									'italic' => esc_html__('Italic', 'goodlayers-core'),
								),
								'default' => 'normal'
							),
							'blog-title-font-weight' => array(
								'title' => esc_html__('Blog Title Font Weight', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Eg. lighter, bold, normal, 300, 400, 600, 700, 800', 'goodlayers-core')
							),
							'blog-title-letter-spacing' => array(
								'title' => esc_html__('Blog Title Letter Spacing', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'blog-title-text-transform' => array(
								'title' => esc_html__('Blog Title Text Transform', 'goodlayers-core'),
								'type' => 'combobox',
								'data-type' => 'text',
								'options' => array(
									'none' => esc_html__('None', 'goodlayers-core'),
									'uppercase' => esc_html__('Uppercase', 'goodlayers-core'),
									'lowercase' => esc_html__('Lowercase', 'goodlayers-core'),
									'capitalize' => esc_html__('Capitalize', 'goodlayers-core'),
								),
								'default' => 'none'
							),
						),
					),
					'shadow' => array(
						'title' => esc_html__('Color/Shadow', 'goodlayers-core'),
						'options' => array(
							'category-background-color' => array(
								'title' => esc_html__('Category Background Color (If Exists)', 'goodlayers-core'),
								'type' => 'colorpicker',
							),
							'tag-color' => array(
								'title' => esc_html__('Tag Color (If Exists)', 'goodlayers-core'),
								'type' => 'colorpicker',
							),
							'tag-hover-color' => array(
								'title' => esc_html__('Tag Hover Border Color (If Exists)', 'goodlayers-core'),
								'type' => 'colorpicker',
							),
							'thumbnail-date-background-color' => array(
								'title' => esc_html__('Thumbnail Date Background Color (If Exists)', 'goodlayers-core'),
								'type' => 'colorpicker',
							),
							'thumbnail-date-color' => array(
								'title' => esc_html__('Thumbnail Date Color (If Exists)', 'goodlayers-core'),
								'type' => 'colorpicker',
							),
							'frame-shadow-size' => array(
								'title' => esc_html__('Shadow Size ( for image/frame )', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'options' => array('x', 'y', 'size'),
								'data-input-type' => 'pixel',
							),
							'frame-shadow-color' => array(
								'title' => esc_html__('Shadow Color ( for image/frame )', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'frame-shadow-opacity' => array(
								'title' => esc_html__('Shadow Opacity ( for image/frame )', 'goodlayers-core'),
								'type' => 'text',
								'default' => '0.2',
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core')
							),
							'enable-move-up-shadow-effect' => array(
								'title' => esc_html__('Move Up Shadow Hover Effect', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable',
								'description' => esc_html__('Only effects the "Column With Frame" style', 'goodlayers-core')
							),
							'move-up-effect-length' => array(
								'title' => esc_html__('Move Up Hover Effect Length', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'condition' => array( 'enable-move-up-shadow-effect' => 'enable' )
							),
							'frame-hover-shadow-size' => array(
								'title' => esc_html__('Shadow Hover Size', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'options' => array('x', 'y', 'size'),
								'data-input-type' => 'pixel',
								'condition' => array( 'enable-move-up-shadow-effect' => 'enable' )
							),
							'frame-hover-shadow-color' => array(
								'title' => esc_html__('Shadow Hover Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'enable-move-up-shadow-effect' => 'enable' )
							),
							'frame-hover-shadow-opacity' => array(
								'title' => esc_html__('Shadow Hover Opacity', 'goodlayers-core'),
								'type' => 'text',
								'default' => '0.2',
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core'),
								'condition' => array( 'enable-move-up-shadow-effect' => 'enable' )
							),
						),
					),

					'spacing' => array(
						'title' => esc_html__('Frame / Spacing', 'goodlayers-core'),
						'options' => array(
							'blog-border-radius' => array(
								'title' => esc_html__('Blog Frame/Thumbnail Border Radius', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'blog-frame-border-size' => array(
								'title' => esc_html__('Blog Frame Border Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'blog-frame-border-color' => array(
								'title' => esc_html__('Blog Frame Border Color', 'goodlayers-core'),
								'type' => 'colorpicker',
							),
							'blog-frame-hover-border-color' => array(
								'title' => esc_html__('Blog Frame Hover Border Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'data-input-type' => 'pixel',
								'description' => esc_html__('Only For Blog Column With Frame Style', 'goodlayers-core')
							),
							'blog-frame-padding' => array(
								'title' => esc_html__('Blog Frame Padding', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'unlink' ),
							),
							'blog-image-overlay-content-padding' => array(
								'title' => esc_html__('Blog Image Overlay Content Padding', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'unlink' ),
							),
							'margin-bottom' => array(
								'title' => esc_html__('Margin Bottom ( Each Blog Post )', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'description' => esc_html__('This option will be omitted for no-space/parallel-space style.', 'goodlayers-core')
							),
							'padding-bottom' => array(
								'title' => esc_html__('Padding Bottom', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => $gdlr_core_item_pdb
							),
						),
					),
					'item-title' => array(
						'title' => esc_html__('Item Title', 'goodlayers-core'),
						'options' => gdlr_core_block_item_option()
					)
				));
			}

			// get the preview for page builder
			static function get_preview( $settings = array() ){
				$content  = self::get_content($settings, true);
				$id = mt_rand(0, 9999);
				
				ob_start();
?><script id="gdlr-core-preview-blog-<?php echo esc_attr($id); ?>" >
if( document.readyState == 'complete' ){
	jQuery(document).ready(function(){
		var blog_preview = jQuery('#gdlr-core-preview-blog-<?php echo esc_attr($id); ?>').parent();
		blog_preview.gdlr_core_flexslider().gdlr_core_isotope().gdlr_core_content_script();
		new gdlr_core_sync_height(blog_preview);
	});
}else{
	jQuery(window).on('load', function(){
		setTimeout(function(){
			var blog_preview = jQuery('#gdlr-core-preview-blog-<?php echo esc_attr($id); ?>').parent();
			blog_preview.gdlr_core_flexslider().gdlr_core_isotope().gdlr_core_content_script();
			new gdlr_core_sync_height(blog_preview);
		}, 300);
		
	});
}
</script><?php	
				$content .= ob_get_contents();
				ob_end_clean();
				
				return $content;
			}			
			
			// get the content from settings
			static function get_content( $settings = array(), $preview = false ){
				global $gdlr_core_item_pdb;
				
				// default variable
				if( empty($settings) ){
					$settings = array(
						'category' => '', 'tag' => '', 'num-fetch' => '9', 'prepend-sticky' => 'disable', 'thumbnail-size' => 'full', 'orderby' => 'date', 'order' => 'desc',
						'blog-style' => 'blog-full', 'excerpt' => 'specify-number', 'excerpt-number' => 55, 'show-read-more' => 'enable', 'column-size' => 20,
						'show-thumbnail' => 'enable',
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}
				
				$settings['meta-option'] = isset($settings['meta-option'])? $settings['meta-option']: array();
				if( $settings['meta-option'] == '' ){
					$settings['meta-option'] = array();
				}else if( !is_array($settings['meta-option']) ){
					$settings['meta-option'] = array_map('trim', explode(',', $settings['meta-option']));
				}
				$settings['blog-style'] = empty($settings['blog-style'])? 'blog-full': $settings['blog-style'];
				$settings['no-space'] = (strpos($settings['blog-style'], 'no-space') !== false)? 'yes': 'no';
				$settings['layout'] = empty($settings['layout'])? 'fitrows': $settings['layout'];
				if( in_array($settings['blog-style'], array('blog-column', 'blog-column-with-frame', 'blog-column-no-space', 'blog-column-hover-bg', 'blog-image', 'blog-image-no-space')) ){
					$settings['has-column'] = 'yes';
				}else if( in_array($settings['blog-style'], array('blog-metro', 'blog-metro-no-space')) ){
					$settings['has-column'] = 'yes';
					$settings['layout'] = 'masonry';
				}else{
					$settings['has-column'] = 'no';
					$settings['column-size'] = 60;
					if( $settings['layout'] == 'masonry' ){
						$settings['layout'] = 'fitrows';
					}else if( $settings['layout'] == 'carousel' ){
						if( !in_array($settings['blog-style'], array('blog-full', 'blog-full-with-frame', 'blog-left-thumbnail', 'blog-right-thumbnail')) ){
							$settings['layout'] = 'fitrows';
						}
					}
				}

				$custom_style  = '';
				if( $settings['blog-style'] == 'blog-column-with-frame' ){
					if( !empty($settings['blog-frame-hover-border-color']) ){
						$custom_style .= '#custom_style_id .gdlr-core-blog-grid.gdlr-core-blog-grid-with-frame:hover{ border-color: ' . $settings['blog-frame-hover-border-color'] . ' !important; }';
					}
					if( !empty($settings['enable-move-up-shadow-effect']) && $settings['enable-move-up-shadow-effect'] == 'enable' ){
						$custom_style_temp = gdlr_core_esc_style(array(
							'background-shadow-size' => empty($settings['frame-hover-shadow-size'])? '': $settings['frame-hover-shadow-size'],
							'background-shadow-color' => empty($settings['frame-hover-shadow-color'])? '': $settings['frame-hover-shadow-color'],
							'background-shadow-opacity' => empty($settings['frame-hover-shadow-opacity'])? '': $settings['frame-hover-shadow-opacity'],
						), false);

						if( !empty($settings['move-up-effect-length']) ){
							$custom_style_temp .= 'transform: translate3d(0, -' . $settings['move-up-effect-length'] . ', 0); ';
						}

						if( !empty($custom_style_temp) ){
							$custom_style .= '#custom_style_id .gdlr-core-move-up-with-shadow:hover{ ' . $custom_style_temp . ' }';
						}
					}
				}else if( $settings['blog-style'] == 'blog-column-hover-bg' ){
					if( !empty($settings['thumbnail-date-color']) ){
						$custom_style .= '#custom_style_id .gdlr-core-blog-column-hover-background .gdlr-core-column-date a{ color:' . $settings['thumbnail-date-color'] . '; }'; 
					}
				}
				if( !empty($settings['carousel-navigation-icon-hover-color']) ){
					$custom_style .= '#custom_style_id .gdlr-core-flexslider-custom-nav i:hover{ color: ' . $settings['carousel-navigation-icon-hover-color'] . ' !important; }';
				}
				if( !empty($settings['tag-color']) ){
					$custom_style .= '#custom_style_id .gdlr-core-blog-full.gdlr-core-style-2-tag .gdlr-core-blog-info-tag a ' .
									 '#custom_style_id .gdlr-core-blog-grid.gdlr-core-style-2-tag .gdlr-core-blog-info-tag a{ color: ' . $settings['tag-color'] . '; }';
				}
				if( !empty($settings['tag-border-color']) ){
					$custom_style .= '#custom_style_id .gdlr-core-blog-full.gdlr-core-style-2-tag .gdlr-core-blog-info-tag a ' .
									 '#custom_style_id .gdlr-core-blog-grid.gdlr-core-style-2-tag .gdlr-core-blog-info-tag a{ border-color: ' . $settings['tag-color'] . '; }';
				}
				if( !empty($settings['tag-hover-color']) ){
					$custom_style .= '#custom_style_id .gdlr-core-blog-full.gdlr-core-style-2-tag .gdlr-core-blog-info-tag a:hover ' .
									 '#custom_style_id .gdlr-core-blog-grid.gdlr-core-style-2-tag .gdlr-core-blog-info-tag a:hover{ color: ' . $settings['tag-hover-color'] . '; }';
				}
				if( !empty($settings['tag-hover-border-color']) ){
					$custom_style .= '#custom_style_id .gdlr-core-blog-full.gdlr-core-style-2-tag .gdlr-core-blog-info-tag a:hover ' .
									 '#custom_style_id .gdlr-core-blog-grid.gdlr-core-style-2-tag .gdlr-core-blog-info-tag a:hover{ border-color: ' . $settings['tag-color'] . '; }';
				}
				if( !empty($custom_style) ){
					if( empty($settings['id']) ){
						global $gdlr_core_blog_id;

						if( $preview ){
							$gdlr_core_blog_id = empty($gdlr_core_blog_id)? array(): $gdlr_core_blog_id;

							// generate unique id so it does not get overwritten in admin area
							$rnd_blog_id = mt_rand(0, 99999);
							while( in_array($rnd_blog_id, $gdlr_core_blog_id) ){
								$rnd_blog_id = mt_rand(0, 99999);
							}
							$gdlr_core_blog_id[] = $rnd_blog_id;
							$settings['id'] = 'gdlr-core-blog-' . $rnd_blog_id;
						}else{
							$gdlr_core_blog_id = empty($gdlr_core_blog_id)? 1: $gdlr_core_blog_id + 1;
							$settings['id'] = 'gdlr-core-blog-' . $gdlr_core_blog_id;
						}
					}

					$custom_style = str_replace('custom_style_id', $settings['id'], $custom_style); 
					if( $preview ){
						$custom_style = '<style>' . $custom_style . '</style>';
					}else{
						gdlr_core_add_inline_style($custom_style);
						$custom_style = '';
					}
				}

				// start printing item
				$extra_class = ' gdlr-core-style-' . $settings['blog-style'];
				if( in_array($settings['blog-style'], array('blog-column', 'blog-column-with-frame', 'blog-column-no-space')) ){
					$extra_class .= ' gdlr-core-' . (empty($settings['blog-column-text-align'])? 'left': $settings['blog-column-text-align']) . '-align';
				}
				
				$title_settings = $settings;
				if( $settings['no-space'] == 'yes' || $settings['layout'] == 'carousel' || $settings['blog-style'] == 'blog-feature' ){
					$title_settings['pdlr'] = false;
					$extra_class .= ' gdlr-core-item-pdlr';
				}
				if( $settings['layout'] == 'carousel' ){
					if( empty($settings['carousel-navigation']) || in_array($settings['carousel-navigation'], array('navigation', 'both')) ){
						$title_settings['carousel'] = 'enable';
					}
				}
				$extra_class .= empty($settings['class'])? '': ' ' . $settings['class'];

				$ret  = '<div class="gdlr-core-blog-item gdlr-core-item-pdb clearfix ' . esc_attr($extra_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';

				// print title
				$ret .= gdlr_core_block_item_title($title_settings);
				
				// pring blog item
				$blog_item = new gdlr_core_blog_item($settings);

				$ret .= $blog_item->get_content();
				
				$ret .= '</div>'; // gdlr-core-blog-item
				$ret .= $custom_style;
				
				return $ret;
			}			
			
		} // gdlr_core_pb_element_blog
	} // class_exists	

	add_shortcode('gdlr_core_blog', 'gdlr_core_blog_shortcode');
	if( !function_exists('gdlr_core_blog_shortcode') ){
		function gdlr_core_blog_shortcode($atts){
			$atts = wp_parse_args($atts, array(
				
			));

			$ret  = '<div class="gdlr-core-blog-shortcode clearfix gdlr-core-item-rvpdlr" >';
			$ret .= gdlr_core_pb_element_blog::get_content($atts);
			$ret .= '</div>';

			return $ret;
		}
	}